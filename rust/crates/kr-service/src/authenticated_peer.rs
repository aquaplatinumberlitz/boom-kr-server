//! Company companion endpoint. Native packets never carry these capabilities
//! or route identifiers. Room owner revalidates account/member state per pump.
use crate::account_bridge::Bridge;
use kr_tunnel::{
    Identity, Link, Secret, control,
    router::{Assignment, Limits, Member, Owner, Router, Scope},
};
use serde::{Deserialize, Serialize, Serializer};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    io,
    net::SocketAddrV4,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};
use tokio::{
    sync::{Semaphore, mpsc, watch},
    task::JoinSet,
};
use zeroize::Zeroizing;

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub bind: SocketAddrV4,
    pub advertise: SocketAddrV4,
    pub max_peers: usize,
    pub route_capacity: usize,
    pub queue_messages: usize,
    pub send_budget: usize,
    pub pending_ms: u64,
    pub lease_seconds: u64,
}
impl Config {
    fn limits(&self) -> io::Result<Limits> {
        if self.bind.port() == 0
            || self.advertise.port() == 0
            || self.advertise.ip().is_unspecified()
            || self.advertise.ip().is_multicast()
            || self.advertise.ip().is_broadcast()
            || !(2..=128).contains(&self.max_peers)
        {
            return Err(io::Error::other("authenticated peer endpoint bounds"));
        }
        let limits = Limits {
            routes: self.route_capacity,
            queue: self.queue_messages,
            sends: self.send_budget,
            pending: Duration::from_millis(self.pending_ms),
            lease: Duration::from_secs(self.lease_seconds),
        };
        Router::new(limits)?;
        Ok(limits)
    }
}
#[derive(Serialize)]
pub struct Offer {
    pub(crate) endpoint: SocketAddrV4,
    pub(crate) certificate_der: Vec<u8>,
    #[serde(serialize_with = "serialize_secret")]
    pub(crate) capability: Secret,
}
fn serialize_secret<S: Serializer>(secret: &Secret, serializer: S) -> Result<S::Ok, S::Error> {
    use std::fmt::Write;
    let mut hex = Zeroizing::new(String::with_capacity(64));
    for byte in secret.expose() {
        write!(&mut *hex, "{byte:02x}").expect("String format");
    }
    serializer.serialize_str(&hex)
}
fn digest(bytes: &[u8; 32]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}
struct Pending {
    owner: Owner,
    until: Instant,
}
struct Peer {
    id: u64,
    data: quinn::Connection,
    control: mpsc::Sender<control::Server>,
}
struct State {
    pending: HashMap<[u8; 32], Pending>,
    peers: HashMap<Owner, Peer>,
    router: Router,
    next_peer: u64,
}
struct Registration {
    hub: Arc<Hub>,
    owner: Owner,
    id: u64,
    connection: quinn::Connection,
}
impl Drop for Registration {
    fn drop(&mut self) {
        if let Ok(mut state) = self.hub.lock()
            && state
                .peers
                .get(&self.owner)
                .is_some_and(|peer| peer.id == self.id)
        {
            state.peers.remove(&self.owner);
            state.router.disconnect(self.owner);
        }
        self.connection.close(0_u8.into(), b"account owner ended");
    }
}
pub(crate) struct Hub {
    config: Config,
    identity: Mutex<Option<Identity>>,
    certificate: Vec<u8>,
    pub(crate) generation: [u8; 16],
    state: Mutex<State>,
    received: AtomicU64,
    accepted: AtomicU64,
    dropped: AtomicU64,
}
impl Hub {
    pub(crate) fn new(config: &Config) -> io::Result<Arc<Self>> {
        let limits = config.limits()?;
        let identity = Identity::ephemeral()?;
        let certificate = identity.certificate.as_ref().to_vec();
        let mut generation = [0; 16];
        getrandom::fill(&mut generation).map_err(io::Error::other)?;
        if generation == [0; 16] {
            return Err(io::Error::other("authenticated peer generation entropy"));
        }
        Ok(Arc::new(Self {
            config: config.clone(),
            identity: Mutex::new(Some(identity)),
            certificate,
            generation,
            state: Mutex::new(State {
                pending: HashMap::new(),
                peers: HashMap::new(),
                router: Router::new(limits)?,
                next_peer: 0,
            }),
            received: AtomicU64::new(0),
            accepted: AtomicU64::new(0),
            dropped: AtomicU64::new(0),
        }))
    }
    fn lock(&self) -> io::Result<std::sync::MutexGuard<'_, State>> {
        self.state
            .lock()
            .map_err(|_| io::Error::other("authenticated peer owner unavailable"))
    }
    pub(crate) fn offer(&self, owner: Owner, ttl: Duration) -> io::Result<Offer> {
        if owner.player == 0
            || owner.session == 0
            || ttl == Duration::ZERO
            || ttl > Duration::from_secs(120)
        {
            return Err(io::Error::other("invalid authenticated peer offer"));
        }
        let mut state = self.lock()?;
        let now = Instant::now();
        state.pending.retain(|_, entry| now < entry.until);
        if state.pending.len() + state.peers.len() >= self.config.max_peers
            || state.peers.contains_key(&owner)
            || state.pending.values().any(|pending| pending.owner == owner)
        {
            return Err(io::Error::other("authenticated peer admission busy"));
        }
        let secret = Secret::random()?;
        let key = digest(secret.expose());
        if state.pending.contains_key(&key) {
            return Err(io::Error::other(
                "authenticated capability entropy collision",
            ));
        }
        state.pending.insert(
            key,
            Pending {
                owner,
                until: now + ttl,
            },
        );
        Ok(Offer {
            endpoint: self.config.advertise,
            certificate_der: self.certificate.clone(),
            capability: secret,
        })
    }
    pub(crate) fn connected(&self, owner: Owner) -> bool {
        self.lock()
            .is_ok_and(|state| state.peers.contains_key(&owner))
    }
    /// Bind before the game service publishes readiness; the future is an owned
    /// listener job and shutdown closes/join every peer worker.
    pub(crate) fn bind(&self) -> io::Result<quinn::Endpoint> {
        let identity = self
            .identity
            .lock()
            .map_err(|_| io::Error::other("peer TLS identity unavailable"))?
            .take()
            .ok_or_else(|| io::Error::other("peer endpoint already bound"))?;
        identity.listen(self.config.bind.into())
    }
    pub(crate) async fn run(
        self: Arc<Self>,
        endpoint: quinn::Endpoint,
        bridge: Bridge,
        mut stop: watch::Receiver<bool>,
    ) -> io::Result<()> {
        let quota = Arc::new(Semaphore::new(self.config.max_peers));
        let mut jobs = JoinSet::new();
        let result = loop {
            tokio::select! {
                _=stop.changed()=>break Ok(()),
                done=jobs.join_next(),if !jobs.is_empty()=>{if let Some(Err(error))=done{break Err(io::Error::other(error));}},
                incoming=endpoint.accept()=>{
                    let Some(incoming)=incoming else{break Ok(());};
                    let Ok(permit)=quota.clone().try_acquire_owned()else{incoming.refuse();continue;};
                    let hub=self.clone();let bridge=bridge.clone();let stop=stop.clone();
                    jobs.spawn(async move {
                        let _permit=permit;
                        let connection=tokio::time::timeout(Duration::from_secs(3),incoming).await.ok().and_then(Result::ok);
                        if let Some(connection)=connection {let _=hub.peer(connection,bridge,stop).await;}
                    });
                }
            }
        };
        endpoint.close(0_u8.into(), b"service shutdown");
        jobs.abort_all();
        while jobs.join_next().await.is_some() {}
        let mut state = self.lock()?;
        state.pending.clear();
        state.peers.clear();
        state.router = Router::new(self.config.limits()?)?;
        result
    }
    async fn peer(
        self: Arc<Self>,
        connection: quinn::Connection,
        bridge: Bridge,
        mut stop: watch::Receiver<bool>,
    ) -> io::Result<()> {
        let (commands, mut control_rx) = mpsc::channel(4);
        let register_connection = connection.clone();
        let register_hub = self.clone();
        let register_bridge = bridge.clone();
        let (mut link, registration) = tokio::select! {
            _=stop.changed()=>return Ok(()),
            value=Link::accept_with_async(connection,|secret|async move {
                let pending={
                    let mut state=register_hub.lock().ok()?;
                    let pending=state.pending.remove(&digest(secret.expose()))?;
                    if Instant::now()>=pending.until||state.peers.contains_key(&pending.owner){return None;}
                    pending
                };
                let owner=pending.owner;
                register_bridge.validate_transport(owner.player,owner.session).await.ok()?;
                let id={let mut state=register_hub.lock().ok()?;
                    if Instant::now()>=pending.until||state.peers.contains_key(&owner){return None;}
                    state.next_peer=state.next_peer.checked_add(1)?;let id=state.next_peer;
                    state.peers.insert(owner,Peer {id,data:register_connection.clone(),control:commands});id
                };
                Some(Registration {hub:register_hub,owner,id,connection:register_connection})
            })=>value?
        };
        let owner = registration.owner;
        let id = registration.id;
        let (mut send, mut recv) = link.take_control()?;
        let (ready, mut ack) = mpsc::channel(4);
        let mut reader_stop = stop.clone();
        let mut reader = tokio::spawn(async move {
            loop {
                let message = tokio::select! {_=reader_stop.changed()=>return Ok::<(),io::Error>(()),result=control::read::<control::Client>(&mut recv)=>result?};
                tokio::select! {_=reader_stop.changed()=>return Ok(()),result=ready.send(message)=>if result.is_err(){return Ok(())}}
            }
        });
        let mut reader_finished = false;
        let mut health = tokio::time::interval(Duration::from_secs(1));
        let result = async { loop {
            tokio::select! {
                _=stop.changed()=>break Ok(()),
                result=&mut reader=>{reader_finished=true;break result.map_err(io::Error::other).and_then(|v|v);},
                _=health.tick()=>{
                    let valid=tokio::select! {_=stop.changed()=>break Ok(()),value=bridge.validate_transport(owner.player,owner.session)=>value};
                    if valid.is_err(){break Err(io::Error::other("companion account lease ended"));}
                },
                command=control_rx.recv()=>{
                    let Some(command)=command else{break Ok(());};
                    if !matches!(tokio::time::timeout(Duration::from_secs(2),control::write(&mut send,&command)).await,Ok(Ok(()))){break Err(io::Error::other("peer control write failed"));}
                },
                message=ack.recv()=>{
                    let Some(control::Client::Ready {scope,bindings})=message else{break Err(io::Error::other("peer readiness stream closed"));};
                    // A stale ACK cannot change current endpoints. Malformed or
                    // foreign ACKs fail the companion owner rather than being guessed.
                    if self.lock()?.router.acknowledge(scope,owner,&bindings,Instant::now()).is_err(){break Err(io::Error::other("peer readiness rejected"));}
                },
                message=link.receive()=>{
                    let packet=match message {Ok(packet)=>packet,Err(e)=>break Err(e)};
                    self.received.fetch_add(1,Ordering::Relaxed);
                    let mut state=self.lock()?;
                    if state.peers.get(&owner).is_none_or(|p|p.id!=id){break Err(io::Error::other("stale authenticated peer owner"));}
                    if state.router.ingest(owner,packet,Instant::now()).is_ok(){self.accepted.fetch_add(1,Ordering::Relaxed);}else{self.dropped.fetch_add(1,Ordering::Relaxed);}
                }
            }
        }}.await;
        if !reader_finished {
            reader.abort();
            let _ = reader.await;
        }
        drop(registration);
        link.close();
        result
    }
    pub(crate) fn prepare(&self, scope: Scope, members: &[Member]) -> io::Result<()> {
        if scope.service != self.generation {
            return Err(io::Error::other("foreign authenticated service generation"));
        }
        let mut state = self.lock()?;
        let mut permits = Vec::new();
        for member in members {
            let peer = state
                .peers
                .get(&member.owner)
                .ok_or_else(|| io::Error::other("companion not connected"))?;
            permits.push((
                member.owner,
                peer.control
                    .clone()
                    .try_reserve_owned()
                    .map_err(|_| io::Error::other("companion control queue full"))?,
            ));
        }
        let assignments: HashMap<Owner, Vec<Assignment>> = state
            .router
            .prepare(scope, members, Instant::now())?
            .into_iter()
            .collect();
        for (owner, permit) in permits {
            permit.send(control::Server::Configure {
                scope,
                routes: assignments[&owner].clone(),
            });
        }
        Ok(())
    }
    pub(crate) fn ready(&self, scope: Scope) -> bool {
        self.lock().is_ok_and(|state| state.router.ready(scope))
    }
    pub(crate) fn endpoints(&self, scope: Scope, owner: Owner) -> io::Result<Vec<(u8, u32, u16)>> {
        self.lock()?.router.endpoints(scope, owner)
    }
    pub(crate) fn activate(&self, scope: Scope, members: &[Member]) -> io::Result<()> {
        self.lock()?.router.activate(scope, members, Instant::now())
    }
    pub(crate) fn retain(&self, scope: Scope, members: &[Member]) -> io::Result<()> {
        self.lock()?.router.retain(scope, members)
    }
    pub(crate) fn retire(&self, scope: Scope) {
        if let Ok(mut state) = self.lock() {
            let owners = state.router.owners(scope);
            state.router.retire(scope);
            // Registry retirement is immediate even if a peer is blocked; a
            // later Configure stops its old snapshot before publishing new153.
            for owner in owners {
                if let Some(peer) = state.peers.get(&owner) {
                    let _ = peer.control.try_send(control::Server::Retire { scope });
                }
            }
        }
    }
    pub(crate) fn pump(
        &self,
        scope: Scope,
        members: &[Member],
    ) -> io::Result<crate::udp_relay::PumpStats> {
        let mut state = self.lock()?;
        let State { router, peers, .. } = &mut *state;
        let stats = router.drain(scope, members, Instant::now(), |owner, packet| {
            let peer = peers
                .get(&owner)
                .ok_or_else(|| io::Error::other("authenticated destination disconnected"))?;
            let encoded = packet.encode()?;
            if peer.data.datagram_send_buffer_space() < encoded.len() {
                return Ok(false);
            }
            peer.data
                .send_datagram(encoded.into())
                .map_err(io::Error::other)?;
            Ok(true)
        })?;
        Ok(crate::udp_relay::PumpStats {
            received: 0,
            accepted: 0,
            dropped: stats.expired,
            sent: stats.sent,
        })
    }
}
