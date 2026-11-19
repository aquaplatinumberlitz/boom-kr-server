//! Native credential ingress only inside pinned TLS13. One connection owns
//! the whole 1P/2P roster; native addresses/echoes never find another owner.
use crate::{
    FixturePin, Validated,
    account_bridge::local::{
        LocalSession,
        native::{self, Driver, Lane, Templates},
    },
    account_worker::Client,
};
use kr_tunnel::prelogin;
use serde::Deserialize;
use std::{
    collections::HashMap,
    fs, io,
    net::{IpAddr, Ipv4Addr, SocketAddrV4},
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};
use tokio::{
    sync::{Mutex as AsyncMutex, Semaphore, mpsc, watch},
    task::JoinSet,
};

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub bind: SocketAddrV4,
    pub certificate: FixturePin,
    pub private_key: PathBuf,
    pub max_connections: usize,
    pub max_unauthenticated: usize,
    pub max_per_ip: usize,
    pub prelogin_seconds: u64,
    pub session_ttl_seconds: i64,
    pub session_idle_seconds: i64,
}
impl Config {
    pub(crate) fn validate(&self) -> io::Result<()> {
        if self.bind.port() == 0
            || self.bind.ip().is_multicast()
            || self.bind.ip().is_broadcast()
            || !(1..=128).contains(&self.max_connections)
            || !(1..=16).contains(&self.max_unauthenticated)
            || self.max_unauthenticated > self.max_connections
            || !(1..=8).contains(&self.max_per_ip)
            || !(10..=180).contains(&self.prelogin_seconds)
            || !(1..=86400).contains(&self.session_ttl_seconds)
            || !(1..=self.session_ttl_seconds).contains(&self.session_idle_seconds)
            || !self.private_key.is_absolute()
            || self
                .private_key
                .components()
                .any(|p| matches!(p, std::path::Component::ParentDir))
        {
            return Err(io::Error::other("prelogin TLS bounds"));
        }
        let file = fs::symlink_metadata(&self.private_key)?;
        let parent = fs::symlink_metadata(
            self.private_key
                .parent()
                .ok_or_else(|| io::Error::other("prelogin key parent"))?,
        )?;
        if !file.is_file()
            || file.permissions().mode() & 0o077 != 0
            || !parent.is_dir()
            || parent.permissions().mode() & 0o077 != 0
            || file.len() == 0
            || file.len() > 8192
        {
            return Err(io::Error::other("prelogin key must be private"));
        }
        Ok(())
    }
    pub(crate) fn open(&self) -> io::Result<quinn::Endpoint> {
        self.validate()?;
        let server =
            prelogin::server_config(self.certificate.load()?, fs::read(&self.private_key)?)?;
        quinn::Endpoint::server(server, self.bind.into())
    }
}
struct Peer {
    since: Instant,
    attempts: usize,
    active: usize,
}
#[derive(Default)]
struct Admission {
    peers: HashMap<IpAddr, Peer>,
}
struct Permit {
    shared: Arc<Mutex<Admission>>,
    peer: IpAddr,
}
impl Admission {
    fn admit(
        shared: &Arc<Mutex<Self>>,
        peer: IpAddr,
        max: usize,
        now: Instant,
    ) -> io::Result<Permit> {
        let mut table = shared
            .lock()
            .map_err(|_| io::Error::other("prelogin admission"))?;
        table.peers.retain(|_, row| {
            row.active != 0 || now.saturating_duration_since(row.since) < Duration::from_secs(60)
        });
        if !table.peers.contains_key(&peer) && table.peers.len() >= 256 {
            return Err(io::Error::other("prelogin peer table bound"));
        }
        let row = table.peers.entry(peer).or_insert(Peer {
            since: now,
            attempts: 0,
            active: 0,
        });
        if now.saturating_duration_since(row.since) >= Duration::from_secs(60) {
            row.since = now;
            row.attempts = 0;
        }
        if row.active >= max || row.attempts >= 5 {
            return Err(io::Error::other("prelogin peer bound"));
        }
        row.active += 1;
        row.attempts += 1;
        Ok(Permit {
            shared: shared.clone(),
            peer,
        })
    }
}
impl Drop for Permit {
    fn drop(&mut self) {
        if let Ok(mut table) = self.shared.lock()
            && let Some(row) = table.peers.get_mut(&self.peer)
        {
            row.active = row.active.saturating_sub(1);
        }
    }
}
struct Close(quinn::Connection);
impl Drop for Close {
    fn drop(&mut self) {
        self.0.close(0u8.into(), b"native transport ended");
    }
}
struct EndpointClose(quinn::Endpoint);
impl Drop for EndpointClose {
    fn drop(&mut self) {
        self.0.close(0u8.into(), b"native listener ended");
    }
}
type Owner = Arc<AsyncMutex<Option<Driver>>>;
struct Resources {
    owner: Owner,
    client: Client,
    templates: Templates,
    hello: Vec<u8>,
    wcp: Vec<u8>,
    ports: Vec<u16>,
    config: Config,
    bytes: AtomicUsize,
    events: mpsc::Sender<serde_json::Value>,
}

async fn stream(
    mut send: quinn::SendStream,
    mut recv: quinn::RecvStream,
    id: u64,
    resources: Arc<Resources>,
) -> io::Result<()> {
    prelogin::accept_header(&mut recv, &resources.ports).await?;
    tokio::time::timeout(Duration::from_secs(2), send.write_all(&resources.hello))
        .await
        .map_err(|_| io::Error::other("native hello deadline"))?
        .map_err(io::Error::other)?;
    let mut sequence = 40;
    let first = match native::read(&mut recv, &mut sequence).await {
        Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => return Ok(()),
        result => result?,
    };
    if first.opcode == 0 {
        tokio::time::timeout(Duration::from_secs(2), send.write_all(&resources.wcp))
            .await
            .map_err(|_| io::Error::other("native WCP deadline"))?
            .map_err(io::Error::other)?;
        let mut extra = [0; 1];
        let _ = tokio::time::timeout(Duration::from_secs(120), recv.read(&mut extra)).await;
        return Ok(());
    }
    let mut lane = match first.opcode {
        1 | 62 => Lane::login(id)?,
        6 => Lane::channel(id)?,
        _ => return Err(io::Error::other("native first opcode")),
    };
    let mut pending = Some(first);
    let result = async {
        loop {
            let input = if let Some(input) = pending.take() {
                input
            } else {
                native::read(&mut recv, &mut sequence).await?
            };
            let size = input.payload_len() + kr_wire::FRAME_OVERHEAD;
            if resources.bytes.fetch_add(size, Ordering::Relaxed) + size > prelogin::BYTE_LIMIT {
                return Err(io::Error::other("native owner byte bound"));
            }
            let mut owner = resources.owner.lock().await;
            if owner.is_none() && matches!(input.opcode, 1 | 62) {
                *owner = Some(Driver::new(
                    LocalSession::new(resources.client.clone(), id)
                        .map_err(|_| io::Error::other("native owner creation"))?,
                    resources.templates.clone(),
                )?);
            }
            let driver = owner
                .as_mut()
                .ok_or_else(|| io::Error::other("native owner absent"))?;
            let opcode = input.opcode;
            driver
                .packet(
                    &mut lane,
                    input,
                    &mut send,
                    resources.config.session_ttl_seconds,
                    Some(resources.config.session_idle_seconds),
                )
                .await?;
            if opcode == 6 {
                let players: Vec<_> = driver
                    .authorities()
                    .iter()
                    .map(|authority| authority.player().wire_id)
                    .collect();
                let _ = resources.events.try_send(
                    serde_json::json!({"event":"native_tls_db_channel","players":players}),
                );
            }
        }
    }
    .await;
    if let Some(driver) = resources.owner.lock().await.as_mut() {
        let retired = driver.disconnected(&lane).await?;
        if !retired
            && result
                .as_ref()
                .is_err_and(|error| error.kind() == io::ErrorKind::UnexpectedEof)
        {
            return Ok(());
        }
    }
    result
}
async fn connection(
    connection: quinn::Connection,
    resources: Arc<Resources>,
    mut stop: watch::Receiver<bool>,
    unauthenticated: tokio::sync::OwnedSemaphorePermit,
) -> io::Result<()> {
    let _close = Close(connection.clone());
    let mut unauthenticated = Some(unauthenticated);
    let mut streams = JoinSet::new();
    let mut number = 0;
    let mut health = tokio::time::interval(Duration::from_millis(250));
    let until = tokio::time::sleep(Duration::from_secs(resources.config.prelogin_seconds));
    tokio::pin!(until);
    let outcome=async{loop{
        if *stop.borrow(){return Ok(());}
        tokio::select!{
            _=stop.changed()=>return Ok(()),
            _=connection.closed()=>return Ok(()),
            _=&mut until,if unauthenticated.is_some()=>return Err(io::Error::other("prelogin lifetime bound")),
            _=health.tick()=>{
                if let Some(driver)=resources.owner.lock().await.as_mut(){
                    driver.inspect().await?;
                    if !driver.authorities().is_empty(){unauthenticated.take();}
                }
            },
            stream=connection.accept_bi()=>{
                let (send,recv)=stream.map_err(io::Error::other)?;number+=1;
                if number>prelogin::STREAM_TOTAL||streams.len()>=prelogin::STREAM_LIMIT as usize{return Err(io::Error::other("native stream admission bound"));}
                let resources=resources.clone();streams.spawn(async move{self::stream(send,recv,number as u64,resources).await});
            },
            result=streams.join_next(),if !streams.is_empty()=>{
                match result{Some(Ok(Ok(())))=>{},Some(Ok(Err(_)))=>return Err(io::Error::other("native stream denied")),Some(Err(_))=>return Err(io::Error::other("native stream task ended")),None=>{}}
            }
        }
    }}.await;
    connection.close(0u8.into(), b"native owner ended");
    streams.abort_all();
    while streams.join_next().await.is_some() {}
    if let Some(driver) = resources.owner.lock().await.take() {
        driver.finish().await?;
    }
    outcome
}
/// A spoofed source cannot hold connection permits or spend another IP's
/// admission rate: unvalidated Initials only get a stateless Retry.
fn validated(incoming: quinn::Incoming) -> Option<quinn::Incoming> {
    if incoming.remote_address_validated() {
        return Some(incoming);
    }
    if let Err(error) = incoming.retry() {
        error.into_incoming().refuse();
    }
    None
}
pub(crate) async fn serve(
    endpoint: quinn::Endpoint,
    config: Config,
    profile: Arc<Validated>,
    client: Client,
    peers: Vec<Ipv4Addr>,
    mut stop: watch::Receiver<bool>,
    events: mpsc::Sender<serde_json::Value>,
) -> io::Result<()> {
    let _close = EndpointClose(endpoint.clone());
    let templates = Templates::from_profile(&profile)?;
    let (hello, wcp) = Templates::startup_frames(&profile);
    let total = Arc::new(Semaphore::new(config.max_connections));
    let unauthenticated = Arc::new(Semaphore::new(config.max_unauthenticated));
    let admission = Arc::new(Mutex::new(Admission::default()));
    let mut owners = JoinSet::new();
    let result=async{loop{
        if *stop.borrow(){return Ok(());}
        tokio::select!{
            _=stop.changed()=>return Ok(()),
            _=owners.join_next(),if !owners.is_empty()=>{},
            incoming=endpoint.accept()=>{
                let Some(incoming)=incoming else{return Err(io::Error::other("prelogin TLS endpoint ended"));};
                let peer=incoming.remote_address().ip();
                if !matches!(peer,IpAddr::V4(ip) if peers.contains(&ip)){incoming.refuse();continue;}
                let Some(incoming)=validated(incoming) else{continue;};
                let Ok(total_permit)=total.clone().try_acquire_owned()else{incoming.refuse();continue;};
                let Ok(unauth)=unauthenticated.clone().try_acquire_owned()else{incoming.refuse();continue;};
                let Ok(peer_permit)=Admission::admit(&admission,peer,config.max_per_ip,Instant::now())else{incoming.refuse();continue;};
                let resources=Arc::new(Resources {owner:Arc::new(AsyncMutex::new(None)),client:client.clone(),templates:templates.clone(),hello:hello.clone(),wcp:wcp.clone(),ports:profile.config.ports.clone(),config:config.clone(),bytes:AtomicUsize::new(0),events:events.clone()});
                let stop=stop.clone();let events=events.clone();owners.spawn(async move{
                    let _total=total_permit;let _peer=peer_permit;
                    let result=tokio::time::timeout(Duration::from_secs(3),incoming).await;
                    if let Ok(Ok(tls))=result{
                        let result=connection(tls,resources,stop,unauth).await;
                        let _=events.try_send(serde_json::json!({"event":"native_tls_owner_ended","ok":result.is_ok()}));
                    }
                });
            }
        }
    }}.await;
    endpoint.close(0u8.into(), b"server stopping");
    if tokio::time::timeout(Duration::from_secs(7), async {
        while owners.join_next().await.is_some() {}
    })
    .await
    .is_err()
    {
        owners.abort_all();
        while owners.join_next().await.is_some() {}
    }
    result
}
#[cfg(test)]
mod tests;
