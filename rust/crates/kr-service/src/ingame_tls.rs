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
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::{
    sync::{Mutex as AsyncMutex, Semaphore, mpsc, watch},
    task::JoinSet,
};
mod lobby_chat;
mod room;

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub bind: SocketAddrV4,
    pub certificate: FixturePin,
    pub private_key: PathBuf,
    pub max_connections: usize,
    pub max_unauthenticated: usize,
    /// Live owners sharing one allowlisted public IP (e.g. NAT).
    pub max_per_ip: usize,
    pub prelogin_seconds: u64,
    #[serde(default = "default_admissions_per_ip_per_minute")]
    pub admissions_per_ip_per_minute: usize,
    pub session_ttl_seconds: i64,
    pub session_idle_seconds: i64,
}
const fn default_admissions_per_ip_per_minute() -> usize {
    5
}
impl Config {
    pub(crate) fn validate(&self) -> io::Result<()> {
        if self.bind.port() == 0
            || self.bind.ip().is_multicast()
            || self.bind.ip().is_broadcast()
            || !(1..=128).contains(&self.max_connections)
            || !(1..=16).contains(&self.max_unauthenticated)
            || self.max_unauthenticated > self.max_connections
            || !(1..=self.max_connections).contains(&self.max_per_ip)
            || !(1..=120).contains(&self.admissions_per_ip_per_minute)
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
        let parent = self
            .private_key
            .parent()
            .ok_or_else(|| io::Error::other("prelogin key parent"))?;
        crate::platform::private_dir(parent)?;
        let file = crate::platform::private_file(&self.private_key)?;
        if file.len() == 0 || file.len() > 8192 {
            return Err(io::Error::other("prelogin key has invalid length"));
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
        attempts: usize,
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
        if row.active >= max || row.attempts >= attempts {
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
    bytes: Mutex<prelogin::TrafficBudget>,
    events: mpsc::Sender<serde_json::Value>,
    hub: Option<Arc<crate::room_service::Hub>>,
    lobby: AsyncMutex<Option<crate::room_service::LocalLobby>>,
    lobby_chat: Arc<lobby_chat::Directory>,
    patched_chat: bool,
    persist_character: bool,
    authenticated: Option<Arc<crate::authenticated_peer::Hub>>,
}

/// The log schema contains only service-chosen labels and numeric protocol
/// metadata. Never serialize an underlying error's text or packet body.
#[derive(Debug)]
struct NativeFailure {
    guard: &'static str,
    opcode: Option<u32>,
    subtype: Option<u8>,
    poll_stage: Option<&'static str>,
    stream_id: Option<u64>,
}
impl std::fmt::Display for NativeFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.guard)
    }
}
impl std::error::Error for NativeFailure {}

fn native_failure(
    error: io::Error,
    fallback: &'static str,
    opcode: Option<u32>,
    subtype: Option<u8>,
) -> io::Error {
    if error.get_ref().is_some_and(|e| e.is::<NativeFailure>()) {
        return error;
    }
    let guard = match error.to_string().as_str() {
        "room slot does not belong to this transport" => "foreign_room_slot",
        "native owner byte bound" => "native_byte_budget",
        "native ancillary bound" => "native_ancillary_budget",
        "native read deadline" => "native_frame_deadline",
        "native checksum" => "native_frame_checksum",
        "native frame bound" => "native_frame_size",
        "native account clock rollback" => "account_clock_rollback",
        "native account worker busy" => "account_worker_busy",
        "native account worker queue expired" => "account_worker_queue_expired",
        "native account worker unavailable" => "account_worker_unavailable",
        "native account storage" => "account_storage",
        "native account owner stale" => "account_owner_stale",
        "native account roster mismatch" => "account_roster_mismatch",
        "native owner denied" => "account_authority",
        "native chat rate bound" => "chat_rate_bound",
        "native session ended" => "account_session_ended",
        "native lobby owner retired" | "native room channel retired" => "retired_membership",
        "report observation phase" => "life_report_phase",
        "unsupported closed-result202 envelope" => "result_tail_shape",
        "active battle exit202 not ported" => "result_tail_phase",
        "incomplete life recipient roster" => "life_recipient_roster",
        "room recipient queue unavailable" => "room_recipient_queue",
        "unsupported Start104 envelope" => "start_shape",
        "unsupported result selector" | "unsupported room selector" => "result_selector",
        "journal missing current outcome" => "journal_missing_outcome",
        "match journal duplicate transition" => "journal_duplicate_finish",
        "match journal round not retired" => "journal_round_not_retired",
        "no pending outcome" | "duplicate/stale outcome completion" => "result_completion",
        "missing validated life170 body" | "missing current life clear" => "life_publication",
        "native room timer queue bound" => "room_timer_queue",
        "native room batch retired" => "retired_room_batch",
        "prelogin lifetime bound" => "prelogin_deadline",
        "native stream admission bound" => "stream_admission",
        _ if error.kind() == io::ErrorKind::UnexpectedEof => "native_stream_eof",
        _ => fallback,
    };
    io::Error::new(
        error.kind(),
        NativeFailure {
            guard,
            opcode,
            subtype,
            poll_stage: None,
            stream_id: None,
        },
    )
}

fn room_poll_failure(error: io::Error, stage: &'static str, opcode: Option<u32>) -> io::Error {
    let mut error = native_failure(error, "room_poll", opcode, None);
    if let Some(failure) = error
        .get_mut()
        .and_then(|error| error.downcast_mut::<NativeFailure>())
    {
        failure.poll_stage = Some(stage);
    }
    error
}

fn room_poll_stream(error: io::Error, id: u64) -> io::Error {
    let mut error = native_failure(error, "room_poll", None, None);
    if let Some(failure) = error
        .get_mut()
        .and_then(|error| error.downcast_mut::<NativeFailure>())
    {
        failure.stream_id = Some(id);
    }
    error
}

fn close_for_session_end(connection: &quinn::Connection, result: &io::Result<()>) {
    if matches!(result, Err(error) if error.get_ref()
        .and_then(|e| e.downcast_ref::<NativeFailure>())
        .is_some_and(|f| f.guard == "account_session_ended"))
    {
        connection.close(
            prelogin::ACCOUNT_SESSION_ENDED.into(),
            b"account session ended",
        );
    }
}

fn owner_ended(result: &io::Result<()>) -> serde_json::Value {
    let mut event = serde_json::json!({"event":"native_tls_owner_ended","ok":result.is_ok()});
    if let Err(error) = result {
        let failure = error
            .get_ref()
            .and_then(|e| e.downcast_ref::<NativeFailure>());
        event["guard"] = serde_json::json!(failure.map_or("owner_lifecycle", |f| f.guard));
        event["opcode"] = serde_json::json!(failure.and_then(|f| f.opcode));
        event["subtype"] = serde_json::json!(failure.and_then(|f| f.subtype));
        if let Some(stage) = failure.and_then(|f| f.poll_stage) {
            event["poll_stage"] = serde_json::json!(stage);
        }
        if let Some(stream_id) = failure.and_then(|f| f.stream_id) {
            event["stream_id"] = serde_json::json!(stream_id);
        }
    }
    event
}

// accept_bi, the current native read and the owner health timer can
// all become ready when the same peer closes QUIC. Match the typed
// close disposition; never reinterpret protocol/DB failures as a close.
fn peer_closed_normally(reason: &quinn::ConnectionError) -> bool {
    matches!(reason, quinn::ConnectionError::ApplicationClosed(close) if close.error_code.into_inner() == 0)
}

async fn stream(
    mut send: quinn::SendStream,
    mut recv: quinn::RecvStream,
    id: u64,
    resources: Arc<Resources>,
    transport: quinn::Connection,
) -> io::Result<()> {
    prelogin::accept_header(&mut recv, &resources.ports)
        .await
        .map_err(|e| native_failure(e, "native_stream_header", None, None))?;
    tokio::time::timeout(Duration::from_secs(2), send.write_all(&resources.hello))
        .await
        .map_err(|_| io::Error::other("native hello deadline"))?
        .map_err(io::Error::other)?;
    let mut sequence = 40;
    let first = match native::read(&mut recv, &mut sequence).await {
        Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => return Ok(()),
        result => result.map_err(|e| native_failure(e, "native_frame_read", None, None))?,
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
        _ => {
            return Err(native_failure(
                io::Error::other("native first opcode"),
                "native_first_opcode",
                Some(first.opcode),
                None,
            ));
        }
    };
    let mut pending = Some(first);
    let mut map_requests = 0;
    let mut room = room::State::default();
    let mut clock = tokio::time::interval(Duration::from_millis(100));
    let mut last = (None, None);
    let result = async {
        loop {
            let input = if let Some(input) = pending.take() {
                input
            } else {
                // Retain this read future across ticks so fragmented native
                // headers are never lost by cancellation and a fresh read.
                let read = native::read(&mut recv, &mut sequence);
                tokio::pin!(read);
                loop {
                    tokio::select! {
                        input = &mut read => {
                            match input {
                                Err(e) if e.kind() == io::ErrorKind::NotConnected
                                    && transport.close_reason().as_ref().is_some_and(peer_closed_normally) =>
                                {
                                    // Own the normal close at the connection level. Do
                                    // not mark the DB roster stale here while its health
                                    // poll can still be running.
                                    return Ok(());
                                }
                                result => break result.map_err(|e|
                                    native_failure(e, "native_frame_read", None, None))?,
                            }
                        }
                        _ = clock.tick(), if lane.in_lobby() => {
                            let mut owner = resources.owner.lock().await;
                            let driver = owner.as_mut().ok_or_else(|| io::Error::other("native owner absent"))?;
                            room.poll(driver, &mut lane, &mut send, &resources, id).await
                                .map_err(|error| room_poll_stream(error, id))?;
                        }
                    }
                }
            };
            let subtype = if input.opcode == 127 {
                input.body().get(..1).and_then(|body|
                    kr_wire::read_uint(body, kr_wire::Width::U8).ok()).map(|v| v as u8)
            } else {
                None
            };
            last = (Some(input.opcode), subtype);
            let size = input.payload_len() + kr_wire::FRAME_OVERHEAD;
            if !resources.bytes.lock().map_err(|_| io::Error::other("native byte budget unavailable"))?
                .allow(size, lane.in_lobby(), Instant::now()) {
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
            if room.packet(&input, driver, &mut lane, &mut send, &resources, id).await.map_err(|e|
                native_failure(e, "room_request", last.0, last.1))? {
                continue;
            }
            if opcode == 528 {
                map_requests += 1;
                if input.payload_len() != 0 || map_requests > 16 {
                    return Err(io::Error::other("native map request shape/budget"));
                }
                let reply = resources
                    .lobby
                    .lock()
                    .await
                    .as_ref()
                    .ok_or_else(|| io::Error::other("native lobby admission absent"))?
                    .map_table()?;
                driver
                    .lobby_reply(&mut lane, &mut send, reply.opcode, &reply.payload)
                    .await?;
                continue;
            }
            if opcode == 67 {
                if !input.lobby_ready() {
                    return Err(io::Error::other("native lobby request shape"));
                }
                let mut lobby = resources.lobby.lock().await;
                let catalog = lobby
                    .as_mut()
                    .ok_or_else(|| io::Error::other("native lobby admission absent"))?
                    .catalog()?;
                driver
                    .enter_lobby(&mut lane, &mut send, &catalog.packets)
                    .await?;
                room.admitted(&catalog, driver, &resources)?;
                let _ = resources
                    .events
                    .try_send(serde_json::json!({"event":"native_local_lobby_ready"}));
                continue;
            }
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
                if let Some(hub) = &resources.hub {
                    let lobby = hub.register_local(driver.authorities())?;
                    *resources.lobby.lock().await = Some(lobby);
                }
                if let Some(hub) = &resources.authenticated {
                    // Only the authenticated channel roster can receive UDP
                    // capabilities, on a separate server TLS stream.
                    let offers = driver.authorities().iter().map(|owner| {
                        hub.offer_local(owner.relay_authority(), Duration::from_secs(30))
                    }).collect::<io::Result<Vec<_>>>()?;
                    tokio::time::timeout(Duration::from_secs(2), async {
                        let mut bytes = zeroize::Zeroizing::new(vec![offers.len() as u8]);
                        for offer in offers {
                            bytes.extend(offer.endpoint.ip().octets());
                            bytes.extend(offer.endpoint.port().to_be_bytes());
                            bytes.extend((offer.certificate_der.len() as u16).to_be_bytes());
                            bytes.extend(offer.certificate_der);
                            bytes.extend(offer.capability.expose());
                        }
                        let mut control = transport.open_uni().await.map_err(io::Error::other)?;
                        control.write_all(&bytes).await.map_err(io::Error::other)?;
                        control.finish().map_err(io::Error::other)?;
                        Ok::<_, io::Error>(())
                    }).await.map_err(|_| io::Error::other("native peer offer delivery deadline"))??;
                }
                let players: Vec<_> = driver
                    .authorities()
                    .iter()
                    .map(|authority| authority.player().wire_id)
                    .collect();
                let connection_id = driver.authorities().first().map(|owner| owner.connection());
                let _ = resources.events.try_send(
                    serde_json::json!({"event":"native_tls_db_channel","connection":connection_id,"players":players}),
                );
            }
        }
    }
    .await
    .map_err(|e| native_failure(e, "native_stage_guard", last.0, last.1));
    if result.is_ok()
        && transport
            .close_reason()
            .as_ref()
            .is_some_and(peer_closed_normally)
    {
        return result;
    }
    if let Some(driver) = resources.owner.lock().await.as_mut() {
        let retired = match driver.retire_disconnected(&lane) {
            Ok(retired) => retired,
            Err(e) if result.is_ok() => {
                return Err(native_failure(e, "account_cleanup", None, None));
            }
            Err(_) => return result,
        };
        if retired && result.is_err() {
            // The health task may observe retirement before it joins this
            // stream. Preserve its original redacted guard in that case.
            let mut event = owner_ended(&result);
            event["event"] = serde_json::json!("native_tls_owned_stream_ended");
            event["stream_id"] = serde_json::json!(id);
            let _ = resources.events.try_send(event);
        }
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
            _=&mut until,if unauthenticated.is_some()=>{
                let has_roster = resources.owner.lock().await.as_ref()
                    .is_some_and(Driver::has_roster);
                if has_roster { unauthenticated.take(); }
                else {
                    connection.close(prelogin::PRELOGIN_EXPIRED.into(), b"prelogin deadline");
                    return Err(io::Error::other("prelogin lifetime bound"));
                }
            },
            _=health.tick()=>{
                let mut owner = resources.owner.lock().await;
                if connection.close_reason().as_ref().is_some_and(peer_closed_normally) {
                    return Ok(());
                }
                if let Some(driver)=owner.as_mut(){
                    driver.inspect().await?;
                    if driver.has_roster(){unauthenticated.take();}
                }
                if resources.lobby.lock().await.as_ref().is_some_and(|lobby|!lobby.current()){
                    return Err(io::Error::other("native lobby owner retired"));
                }
            },
            stream=connection.accept_bi()=>{
                let (send,recv) = match stream {
                    Err(e) if peer_closed_normally(&e) => return Ok(()),
                    result => result.map_err(io::Error::other)?,
                };
                number+=1;
                if number>prelogin::STREAM_TOTAL||streams.len()>=prelogin::STREAM_LIMIT as usize{return Err(io::Error::other("native stream admission bound"));}
                let id=crate::transport_ids::next()?;let resources=resources.clone();let transport=connection.clone();streams.spawn(async move{self::stream(send,recv,id,resources,transport).await});
            },
            result=streams.join_next(),if !streams.is_empty()=>{
                match result{Some(Ok(Ok(())))=>{},Some(Ok(Err(error)))=>return Err(native_failure(error,"native_stream",None,None)),Some(Err(_))=>return Err(io::Error::other("native stream task ended")),None=>{}}
            }
        }
    }}.await.map_err(|e| native_failure(e,"owner_lifecycle",None,None));
    close_for_session_end(&connection, &outcome);
    connection.close(0u8.into(), b"native owner ended");
    streams.abort_all();
    while streams.join_next().await.is_some() {}
    resources.lobby.lock().await.take();
    if let Some(driver) = resources.owner.lock().await.take() {
        if let Some(hub) = &resources.authenticated {
            for owner in driver.authorities() {
                hub.retire_local(&owner.relay_authority());
            }
        }
        let cleanup = driver
            .finish()
            .await
            .map_err(|e| native_failure(e, "account_cleanup", None, None));
        return outcome.and(cleanup);
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
    let lobby_chat = Arc::new(lobby_chat::Directory::default());
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
                let Ok(peer_permit)=Admission::admit(&admission,peer,config.max_per_ip,config.admissions_per_ip_per_minute,Instant::now())else{incoming.refuse();continue;};
                let resources=Arc::new(Resources {owner:Arc::new(AsyncMutex::new(None)),client:client.clone(),templates:templates.clone(),hello:hello.clone(),wcp:wcp.clone(),ports:profile.config.ports.clone(),config:config.clone(),bytes:Mutex::new(prelogin::TrafficBudget::new(prelogin::BYTE_LIMIT,64*1024)),events:events.clone(),hub:profile.bound.as_ref().and_then(|bound|bound.managed.clone()),lobby:AsyncMutex::new(None),lobby_chat:lobby_chat.clone(),patched_chat:profile.config.diagnostic_two_lobby.as_ref().and_then(|two|two.account_admission.as_ref()).is_some_and(|account|account.patched_949x_names),persist_character:profile.config.diagnostic_two_lobby.as_ref().and_then(|two|two.account_admission.as_ref()).is_some_and(|account|account.persist_character),authenticated:profile.bound.as_ref().and_then(|bound|bound.authenticated.clone())});
                let stop=stop.clone();let events=events.clone();owners.spawn(async move{
                    let _total=total_permit;let _peer=peer_permit;
                    let result=tokio::time::timeout(Duration::from_secs(3),incoming).await;
                    if let Ok(Ok(tls))=result{
                        let result=connection(tls,resources,stop,unauth).await;
                        let _=events.try_send(owner_ended(&result));
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
