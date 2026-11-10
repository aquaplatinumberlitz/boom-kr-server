//! Bounded diagnostic KR00967 transport; local account backend has no native adapter.
#![forbid(unsafe_code)]

pub mod account_worker;
pub mod accounts;
pub mod binding;
mod bombs;
mod life;
pub mod lobby;
mod logging;
mod pickups;
pub mod room;
mod room_io;
mod room_wire;
mod terrain;

use kr_wire::{HEADER_LEN, Receiver, decode, encode};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fmt::Write as _;
use std::{
    collections::{HashMap, HashSet},
    io,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::{Semaphore, mpsc, watch},
    task::JoinSet,
    time::timeout,
};

fn sha256_hex(data: &[u8]) -> String {
    let mut out = String::with_capacity(64);
    for byte in Sha256::digest(data) {
        write!(&mut out, "{byte:02x}").expect("String formatting");
    }
    out
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FixturePin {
    pub path: PathBuf,
    pub bytes: usize,
    pub sha256: String,
}
impl FixturePin {
    fn load(&self) -> io::Result<Vec<u8>> {
        if self.bytes > 65_536
            || self.sha256.len() != 64
            || !self
                .sha256
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
        {
            return Err(io::Error::other("invalid fixture pin"));
        }
        if std::fs::metadata(&self.path)?.len() != self.bytes as u64 {
            return Err(io::Error::other("fixture size mismatch"));
        }
        let data = std::fs::read(&self.path)?;
        if sha256_hex(&data) != self.sha256 {
            return Err(io::Error::other("fixture SHA256 mismatch"));
        }
        Ok(data)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LobbyPacketPin {
    pub fixture: FixturePin,
    pub sequence: Option<u32>,
    pub opcode: u32,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LocalLobbyConfig {
    pub packets: HashMap<String, LobbyPacketPin>,
    pub own_header: FixturePin,
}
#[derive(Clone)]
struct Reply {
    header: [u8; HEADER_LEN],
    opcode: u32,
    payload: Vec<u8>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub schema: String,
    pub diagnostic_login_error_only: bool,
    pub bind: Ipv4Addr,
    pub allowed_peers: Vec<Ipv4Addr>,
    pub ports: Vec<u16>,
    pub max_connections: usize,
    pub idle_timeout_seconds: u64,
    pub write_timeout_ms: u64,
    pub bootstrap: FixturePin,
    pub login_error: FixturePin,
    pub wcp_second: FixturePin,
    #[serde(default)]
    pub diagnostic_local_lobby: Option<LocalLobbyConfig>,
    #[serde(default)]
    pub diagnostic_two_lobby: Option<TwoLobbyConfig>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticActorConfig {
    pub peer: Ipv4Addr,
    pub identity: u32,
    pub name: String,
    pub profile: LocalLobbyConfig,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TwoLobbyConfig {
    pub wcp_ports: Vec<u16>,
    pub login_port: u16,
    #[serde(default)]
    pub login_candidate_ports: Vec<u16>,
    pub channel_port: u16,
    pub actors: Vec<DiagnosticActorConfig>,
    #[serde(default)]
    pub fixed_room: Option<room_io::Config>,
}
struct ActorProfile {
    identity: u32,
    name: Vec<u8>,
    packets: HashMap<String, Reply>,
}
struct BoundProfiles {
    actors: HashMap<Ipv4Addr, ActorProfile>,
    registry: binding::Shared,
    room: Option<room_io::Shared>,
}
pub struct Validated {
    config: Config,
    bootstrap: Vec<u8>,
    error_header: [u8; HEADER_LEN],
    wcp_second: Vec<u8>,
    lobby: Option<HashMap<String, Reply>>,
    bound: Option<BoundProfiles>,
}
pub fn load_config(path: &Path) -> io::Result<Validated> {
    if std::fs::metadata(path)?.len() > 65_536 {
        return Err(io::Error::other("config too large"));
    }
    let config: Config = serde_json::from_slice(&std::fs::read(path)?).map_err(io::Error::other)?;
    let two = config.diagnostic_two_lobby.is_some();
    let modes = usize::from(config.diagnostic_login_error_only)
        + usize::from(config.diagnostic_local_lobby.is_some())
        + usize::from(two);
    if config.schema != "kr-rust-diagnostic-v1"
        || modes != 1
        || !(config.bind.is_loopback() || (two && config.bind.is_private()))
        || config.allowed_peers.is_empty()
        || config
            .allowed_peers
            .iter()
            .any(|p| !(p.is_loopback() || (two && p.is_private())))
        || config.ports.is_empty()
        || config.ports.len() > 32
        || config.ports.contains(&0)
        || config.ports.iter().copied().collect::<HashSet<_>>().len() != config.ports.len()
        || !(1..=64).contains(&config.max_connections)
        || !(1..=120).contains(&config.idle_timeout_seconds)
        || !(1..=10_000).contains(&config.write_timeout_ms)
    {
        return Err(io::Error::other("unsupported diagnostic configuration"));
    }
    let bootstrap = config.bootstrap.load()?;
    let error = config.login_error.load()?;
    let wcp_second = config.wcp_second.load()?;
    // The service implements one native-tested KR00967 bootstrap/WCP profile.
    // SHA pin acceptance is a prelisten design guard, not native error behavior.
    if config.bootstrap.sha256 != "75be47b3067e2b1e9d2cf99b5a7eb7f949a074482409fb3eafc1a553cec53cd1"
        || config.wcp_second.sha256
            != "5930ad297fc2c1540d8cbd894e88d3c1d2831bd4565dfb8ad96644568b87ad38"
    {
        return Err(io::Error::other(
            "unsupported KR00967 bootstrap/WCP profile",
        ));
    }

    if wcp_second.is_empty() {
        return Err(io::Error::other("empty WCP fixture"));
    }
    if bootstrap.len() != 15 || error.len() != 39 {
        return Err(io::Error::other("unsupported fixture profile"));
    }
    let packet = decode(&error, 43).map_err(io::Error::other)?;
    if packet.opcode != 4 || packet.payload != [0x5b] {
        return Err(io::Error::other("expected pinned KR result1"));
    }
    let error_header = error[..HEADER_LEN]
        .try_into()
        .expect("validated 39-byte fixture");
    if encode(&error_header, 4, &[0x5b], 43).map_err(io::Error::other)? != error {
        return Err(io::Error::other(
            "result1 encoding disagrees with native-tested fixture",
        ));
    }
    let lobby = config
        .diagnostic_local_lobby
        .as_ref()
        .map(|local| load_lobby(local, 1, b"KRLocal"))
        .transpose()?;
    let bound = if let Some(two) = &config.diagnostic_two_lobby {
        let ports = [two.login_port, two.channel_port];
        if ports.into_iter().collect::<HashSet<_>>().len() != 2
            || two
                .login_candidate_ports
                .iter()
                .any(|p| !config.ports.contains(p))
            || two
                .login_candidate_ports
                .iter()
                .copied()
                .collect::<HashSet<_>>()
                .len()
                != two.login_candidate_ports.len()
            || two.wcp_ports.is_empty()
            || two.wcp_ports.iter().copied().collect::<HashSet<_>>().len() != two.wcp_ports.len()
            || two.wcp_ports.iter().any(|p| !config.ports.contains(p))
            || ports.iter().any(|p| !config.ports.contains(p))
            || !matches!(two.actors.len(), 2 | 4)
            || config.allowed_peers.len() != two.actors.len()
        {
            return Err(io::Error::other("invalid bounded diagnostic endpoints"));
        }
        let mut actors = HashMap::new();
        let mut identities = HashSet::new();
        for actor in &two.actors {
            let expected = match actor.identity {
                1 => "KRLocal",
                2 => "KRGuest",
                3 if two.actors.len() == 4 => "KRNew",
                4 if two.actors.len() == 4 => "KRPeer4",
                _ => return Err(io::Error::other("unsupported diagnostic identity")),
            };
            if actor.name != expected
                || !config.allowed_peers.contains(&actor.peer)
                || !identities.insert(actor.identity)
            {
                return Err(io::Error::other("invalid diagnostic actor binding"));
            }
            let packets = load_lobby(&actor.profile, actor.identity, actor.name.as_bytes())?;
            if actors
                .insert(
                    actor.peer,
                    ActorProfile {
                        identity: actor.identity,
                        name: actor.name.as_bytes().to_vec(),
                        packets,
                    },
                )
                .is_some()
            {
                return Err(io::Error::other("duplicate diagnostic endpoint"));
            }
        }
        let room = two
            .fixed_room
            .as_ref()
            .map(|cfg| room_io::load(cfg, &actors))
            .transpose()?;
        Some(BoundProfiles {
            actors,
            registry: binding::Shared::default(),
            room,
        })
    } else {
        None
    };
    Ok(Validated {
        config,
        bootstrap,
        error_header,
        wcp_second,
        lobby,
        bound,
    })
}

fn load_lobby(
    local: &LocalLobbyConfig,
    identity: u32,
    name: &[u8],
) -> io::Result<HashMap<String, Reply>> {
    let mut packets = HashMap::new();
    let own_header_raw = local.own_header.load()?;
    let own_template = decode(&own_header_raw, 58).map_err(io::Error::other)?;
    if own_template.opcode != 106 || own_template.payload.len() != 190 + 2 * name.len() {
        return Err(io::Error::other("unsupported own291 header template"));
    }
    let own_header: [u8; HEADER_LEN] = own_header_raw[..HEADER_LEN].try_into().expect("decoded106");
    let expected = [
        ("auth_A", 4, 130 + name.len()),
        ("netinfo", 15, 31),
        ("flags", 62, 3),
        ("redirect", 7, 14),
        ("channel", 17, 37),
        ("ready", 241, 31),
        ("roomlist_A", 91, 2),
        ("channel_list", 14, 44),
        ("own291", 291, 121 + name.len()),
    ];
    if local.packets.len() != expected.len() {
        return Err(io::Error::other("unsupported local lobby packet set"));
    }
    for (alias, opcode, size) in expected {
        let pin = local
            .packets
            .get(alias)
            .ok_or_else(|| io::Error::other("missing lobby packet"))?;
        if pin.opcode != opcode {
            return Err(io::Error::other("lobby opcode mismatch"));
        }
        let raw = pin.fixture.load()?;
        let (header, payload) = if let Some(sequence) = pin.sequence {
            let packet = decode(&raw, sequence).map_err(io::Error::other)?;
            if packet.opcode != opcode {
                return Err(io::Error::other("lobby fixture opcode mismatch"));
            }
            (
                raw[..HEADER_LEN].try_into().expect("decodedframe"),
                packet.payload,
            )
        } else {
            // own291 is an explicit body-only pin derived from the measured
            // zero-profile builder. Reuse only opaque auth template header.
            if alias != "own291" {
                return Err(io::Error::other("unexpected body-only pin"));
            }
            (own_header, raw)
        };
        if payload.len() != size {
            return Err(io::Error::other("lobby body size mismatch"));
        }
        packets.insert(
            alias.to_owned(),
            Reply {
                header,
                opcode,
                payload,
            },
        );
    }
    let auth = &packets["auth_A"].payload;
    if auth[0] != 0x5a
        || kr_wire::read_uint(&auth[25..29], kr_wire::Width::U32).map_err(io::Error::other)?
            != identity as u64
    {
        return Err(io::Error::other("diagnostic auth identity mismatch"));
    }
    let size =
        kr_wire::read_uint(&auth[29..31], kr_wire::Width::U16).map_err(io::Error::other)? as usize;
    if size != name.len() || lobby::decode_bytes(&auth[31..31 + size])? != name {
        return Err(io::Error::other("diagnostic auth name mismatch"));
    }
    Ok(packets)
}

type Log = mpsc::Sender<Value>;
async fn emit(log: &Log, value: Value) -> io::Result<()> {
    timeout(Duration::from_millis(200), log.send(value))
        .await
        .map_err(|_| io::Error::other("log backpressure"))?
        .map_err(|_| io::Error::other("log writer unavailable"))
}
fn stamp(start: Instant, mut value: Value) -> Value {
    value["elapsed_ns"] = json!(start.elapsed().as_nanos().min(u64::MAX as u128) as u64);
    value
}
async fn state(out: &Path, ready: bool, stopped: bool) -> io::Result<()> {
    let value = json!({"schema":"kr-rust-diagnostic-v1", "ready":ready, "stopped":stopped, "tcp_events":[], "errors":[]});
    tokio::fs::write(
        out.join("service-state.tmp"),
        serde_json::to_vec(&value).map_err(io::Error::other)?,
    )
    .await?;
    tokio::fs::rename(
        out.join("service-state.tmp"),
        out.join("service-state.json"),
    )
    .await
}

async fn connection(
    mut socket: TcpStream,
    id: u64,
    profile: Arc<Validated>,
    log: Log,
    mut stop: watch::Receiver<bool>,
    start: Instant,
) -> io::Result<()> {
    if profile.bound.is_some() {
        return bound_connection(socket, id, &profile, log, stop, start).await;
    }
    socket.set_nodelay(true)?;
    let write_limit = Duration::from_millis(profile.config.write_timeout_ms);
    timeout(write_limit, socket.write_all(&profile.bootstrap))
        .await
        .map_err(|_| io::Error::other("hello write timeout"))??;
    emit(&log, stamp(start,json!({"event":"hello_sent","connection":id,"bytes":15,"sha256":profile.config.bootstrap.sha256}))).await?;
    // CP181 diagnostic mapping: the first two transports are the known WCP
    // bootstrap path. Its opaque reply is pinned; do not parse it as mode2.
    // This arrival mapping is not multi-account authentication or production routing.
    if id <= 2 {
        let mut bytes_read = 0usize;
        let mut replied = false;
        let mut data = [0; 4096];
        loop {
            let size = tokio::select! {
                _=stop.changed()=>return Ok(()),
                value=timeout(Duration::from_secs(profile.config.idle_timeout_seconds),socket.read(&mut data))=>value.map_err(|_|io::Error::other("WCP read timeout"))??,
            };
            if size == 0 {
                return Ok(());
            }
            bytes_read += size;
            if bytes_read > 65536 {
                return Err(io::Error::other("WCP diagnostic receive bound"));
            }
            if id == 2 && !replied {
                timeout(write_limit, socket.write_all(&profile.wcp_second))
                    .await
                    .map_err(|_| io::Error::other("WCP reply timeout"))??;
                replied = true;
                emit(&log,stamp(start,json!({"event":"wcp_reply_sent","connection":id,"bytes":profile.wcp_second.len(),"sha256":profile.config.wcp_second.sha256,"policy":"CP181_diagnostic_first_two_transports"}))).await?;
            }
        }
    }
    if let Some(packets) = &profile.lobby {
        let flow = match id {
            3 => lobby::Flow::login(),
            4 => lobby::Flow::channel(),
            _ => return Err(io::Error::other("diagnostic local lobby transport limit")),
        };
        return local_lobby_connection(
            socket, id, &profile, packets, flow, None, None, log, stop, start,
        )
        .await;
    }
    let mut receive = Receiver::new(40);
    let mut rejected = false;
    let mut ancillary_count = 0usize;
    let mut chunk = [0; 4096];
    loop {
        while let Some(packet) = receive.next_packet().map_err(io::Error::other)? {
            // Deliberately omit raw auth body, token and body digest from logs.
            emit(&log,stamp(start,json!({"event":"packet_received","connection":id,"opcode":packet.opcode,"sequence":packet.sequence,"payload_bytes":packet.payload.len()}))).await?;
            // CP181 login capture carries ancillary60/body20 and15/body31.
            // Python leaves these unanswered. Preserve that narrow policy,
            // without claiming their meaning or treating them as credentials.
            if rejected && matches!((packet.opcode, packet.payload.len()), (60, 20) | (15, 31)) {
                ancillary_count += 1;
                if ancillary_count > 64 {
                    return Err(io::Error::other("ancillary packet budget"));
                }
                emit(&log,stamp(start,json!({"event":"ancillary_ignored","connection":id,"opcode":packet.opcode,"payload_bytes":packet.payload.len(),"policy":"CP181_observed_shapes_no_reply"}))).await?;
                continue;
            }
            if rejected || packet.opcode != 1 {
                return Err(io::Error::other("unsupported packet/stage"));
            }
            let frame = encode(&profile.error_header, 4, &[0x5b], 43).map_err(io::Error::other)?;
            timeout(write_limit, socket.write_all(&frame))
                .await
                .map_err(|_| io::Error::other("result write timeout"))??;
            rejected = true;
            emit(&log,stamp(start,json!({"event":"login_error_sent","connection":id,"sequence":43,"opcode":4,"bytes":frame.len(),"sha256":sha256_hex(&frame),"policy":"diagnostic_result1_no_authentication"}))).await?;
        }
        let capacity = chunk.len().min(receive.remaining_capacity());
        if capacity == 0 {
            return Err(io::Error::other("receive buffer exhausted"));
        }
        let size = tokio::select! {
            _ = stop.changed() => return Ok(()),
            value = timeout(Duration::from_secs(profile.config.idle_timeout_seconds),socket.read(&mut chunk[..capacity])) => value.map_err(|_|io::Error::other("read idle timeout"))??,
        };
        if size == 0 {
            return receive.finish().map_err(io::Error::other);
        }
        receive.push(&chunk[..size]).map_err(io::Error::other)?;
    }
}

#[allow(clippy::too_many_arguments)]
async fn local_lobby_connection(
    mut socket: TcpStream,
    id: u64,
    profile: &Validated,
    packets: &HashMap<String, Reply>,
    mut flow: lobby::Flow,
    lease: Option<binding::Lease>,
    initial: Option<(Receiver, kr_wire::Packet)>,
    log: Log,
    mut stop: watch::Receiver<bool>,
    start: Instant,
) -> io::Result<()> {
    let (mut rx, mut initial_packet) = match initial {
        Some((rx, packet)) => (rx, Some(packet)),
        None => (Receiver::new(40), None),
    };
    let mut room_session: Option<room_io::Session> = None;
    let mut pending: Vec<room_io::Batch> = Vec::new();
    let mut send_sequence = 43u32;
    let mut buf = [0; 4096];
    let mut read_deadline =
        Instant::now() + Duration::from_secs(profile.config.idle_timeout_seconds);
    loop {
        while let Some(packet) = if let Some(packet) = initial_packet.take() {
            Some(packet)
        } else {
            rx.next_packet().map_err(io::Error::other)?
        } {
            if let Some(session) = &room_session
                && room_io::Session::handles(packet.opcode)
            {
                let observation = session.packet(packet.opcode, &packet.payload)?;
                emit(&log,stamp(start,json!({"event":"room_request","connection":id,"identity":flow.identity(),"opcode":packet.opcode,"sequence":packet.sequence,"payload_bytes":packet.payload.len(),"dropped_after_abort":observation.dropped_after_abort,"pickup":observation.pickup,"bomb":observation.bomb,"terrain_notice":observation.terrain_notice,"life":observation.life}))).await?;
                continue;
            }
            let aliases = flow.packet(packet.opcode, &packet.payload)?;
            if packet.opcode == 6
                && let Some(lease) = &lease
            {
                lease.bind_channel()?;
            }
            emit(&log,stamp(start,json!({"event":"local_lobby_request","connection":id,"opcode":packet.opcode,"sequence":packet.sequence,"payload_bytes":packet.payload.len(),"stage":format!("{:?}",flow.stage)}))).await?;
            for alias in aliases {
                let reply = &packets[alias];
                let frame = encode(&reply.header, reply.opcode, &reply.payload, send_sequence)
                    .map_err(io::Error::other)?;
                timeout(
                    Duration::from_millis(profile.config.write_timeout_ms),
                    socket.write_all(&frame),
                )
                .await
                .map_err(|_| io::Error::other("lobby reply timeout"))??;
                emit(&log,stamp(start,json!({"event":"local_lobby_reply","connection":id,"alias":alias,"opcode":reply.opcode,"sequence":send_sequence,"bytes":frame.len(),"sha256":sha256_hex(&frame),"policy":"diagnostic_KRLocal_not_authentication"}))).await?;
                if alias == "redirect"
                    && let Some(lease) = &lease
                {
                    lease.redirected()?;
                }
                send_sequence = send_sequence.wrapping_add(3);
            }
            if packet.opcode == 67
                && flow.stage == lobby::Stage::Lobby
                && let Some(shared) = profile.bound.as_ref().and_then(|b| b.room.as_ref())
            {
                room_session = Some(room_io::Session::open(shared, flow.identity(), id)?);
            }
        }
        if let Some(session) = &room_session {
            session.tick()?;
        }
        pending.sort_by_key(|batch| batch.due);
        while pending
            .first()
            .is_some_and(|batch| batch.due <= Instant::now())
        {
            let batch = pending.remove(0);
            let Some(session) = &room_session else {
                return Err(io::Error::other("missing room session"));
            };
            if !session.valid(&batch) {
                emit(&log,stamp(start,json!({"event":"room_queued_dropped","connection":id,"reason":"membership_phase_generation"}))).await?;
                continue;
            }
            for (alias, reply) in &batch.packets {
                if !session.valid(&batch) {
                    return Err(io::Error::other("room membership changed during batch"));
                }
                let Some(reply) = session.project(reply)? else {
                    continue;
                };
                let frame = encode(&reply.header, reply.opcode, &reply.payload, send_sequence)
                    .map_err(io::Error::other)?;
                timeout(
                    Duration::from_millis(profile.config.write_timeout_ms),
                    socket.write_all(&frame),
                )
                .await
                .map_err(|_| io::Error::other("room reply timeout"))??;
                emit(&log,stamp(start,json!({"event":"room_reply","connection":id,"identity":flow.identity(),"alias":alias,"opcode":reply.opcode,"sequence":send_sequence,"bytes":frame.len(),"sha256":sha256_hex(&frame)}))).await?;
                send_sequence = send_sequence.wrapping_add(3);
            }
            session.finish(&batch)?;
            if batch.setup.is_some() {
                emit(&log,stamp(start,json!({"event":"room_setup_complete","connection":id,"identity":flow.identity()}))).await?;
            }
        }
        let cap = buf.len().min(rx.remaining_capacity());
        if cap == 0 {
            return Err(io::Error::other("lobby receive bound"));
        }
        let deadline = pending
            .first()
            .map(|b| b.due)
            .unwrap_or_else(|| Instant::now() + Duration::from_secs(3600));
        let deadline = if room_session.is_some() {
            deadline.min(Instant::now() + Duration::from_millis(100))
        } else {
            deadline
        };
        let n = tokio::select! {
            _=stop.changed()=>return Ok(()),
            _=tokio::time::sleep_until(deadline.into()),if !pending.is_empty() || room_session.is_some()=>continue,
            batch=async {match &mut room_session {Some(session)=>session.inbox.recv().await,None=>std::future::pending().await}}=>{
                let batch=batch.ok_or_else(||io::Error::other("room inbox closed"))?;
                if pending.len()>=16 {return Err(io::Error::other("room timer queue bound"));}
                pending.push(batch);continue;
            },
            v=tokio::time::timeout_at(read_deadline.into(),socket.read(&mut buf[..cap]))=>v.map_err(|_|io::Error::other("lobby read idle timeout"))??
        };
        if n == 0 {
            return rx.finish().map_err(io::Error::other);
        }
        read_deadline = Instant::now() + Duration::from_secs(profile.config.idle_timeout_seconds);
        rx.push(&buf[..n]).map_err(io::Error::other)?;
    }
}

#[allow(clippy::too_many_arguments)]
async fn bound_connection(
    mut socket: TcpStream,
    id: u64,
    profile: &Validated,
    log: Log,
    mut stop: watch::Receiver<bool>,
    start: Instant,
) -> io::Result<()> {
    let bound = profile.bound.as_ref().expect("validated bound mode");
    let peer = match socket.peer_addr()?.ip() {
        IpAddr::V4(ip) => ip,
        _ => return Err(io::Error::other("IPv4 diagnostic only")),
    };
    let actor = bound
        .actors
        .get(&peer)
        .ok_or_else(|| io::Error::other("unmapped diagnostic actor"))?;
    let endpoints = profile
        .config
        .diagnostic_two_lobby
        .as_ref()
        .expect("validated endpoints");
    let port = socket.local_addr()?.port();
    socket.set_nodelay(true)?;
    let write_limit = Duration::from_millis(profile.config.write_timeout_ms);
    timeout(write_limit, socket.write_all(&profile.bootstrap))
        .await
        .map_err(|_| io::Error::other("hello write timeout"))??;
    let mut rx = Receiver::new(40);
    let mut buf = [0; 4096];
    loop {
        if let Some(packet) = rx.next_packet().map_err(io::Error::other)? {
            emit(&log,stamp(start,json!({"event":"diagnostic_first_request","connection":id,"opcode":packet.opcode,"payload_bytes":packet.payload.len(),"local_port":port}))).await?;
            let kind = match (packet.opcode, packet.payload.len()) {
                (0, 28) if endpoints.wcp_ports.contains(&port) => binding::Kind::Wcp,
                (1, _)
                    if port == endpoints.login_port
                        || endpoints.login_candidate_ports.contains(&port) =>
                {
                    binding::Kind::Login
                }
                (6, _) if port == endpoints.channel_port => binding::Kind::Channel,
                _ => {
                    return Err(io::Error::other(
                        "unsupported diagnostic first request/endpoint",
                    ));
                }
            };
            let lease = binding::Registry::claim(&bound.registry, peer, id, kind)?;
            emit(&log,stamp(start,json!({"event":"diagnostic_binding","connection":id,"identity":actor.identity,"transport":format!("{:?}",kind),"first_opcode":packet.opcode,"first_payload_bytes":packet.payload.len(),"policy":"operator_endpoint_request_stage_not_authentication"}))).await?;
            if kind == binding::Kind::Wcp {
                // CP006 native type0/body28; WCP destination ports vary (CP103).
                // Empty hello-only sockets claim no actor/session. No accept counter.
                if rx.next_packet().map_err(io::Error::other)?.is_some() {
                    return Err(io::Error::other("extra diagnostic WCP request"));
                }
                timeout(write_limit, socket.write_all(&profile.wcp_second))
                    .await
                    .map_err(|_| io::Error::other("WCP write timeout"))??;
                emit(&log,stamp(start,json!({"event":"wcp_reply_sent","connection":id,"bytes":profile.wcp_second.len(),"sha256":profile.config.wcp_second.sha256,"policy":"CP006_type0_body28_request_not_arrival"}))).await?;
                {
                    let n = tokio::select! {_=stop.changed()=>return Ok(()),v=timeout(Duration::from_secs(profile.config.idle_timeout_seconds),socket.read(&mut buf))=>v.map_err(|_|io::Error::other("WCP idle timeout"))??};
                    if n == 0 {
                        return rx.finish().map_err(io::Error::other);
                    }
                    return Err(io::Error::other("extra diagnostic WCP bytes"));
                }
            }
            let flow = lobby::Flow::for_identity(
                kind == binding::Kind::Channel,
                actor.identity,
                &actor.name,
            );
            return local_lobby_connection(
                socket,
                id,
                profile,
                &actor.packets,
                flow,
                Some(lease),
                Some((rx, packet)),
                log,
                stop,
                start,
            )
            .await;
        }
        let cap = buf.len().min(rx.remaining_capacity());
        if cap == 0 {
            return Err(io::Error::other("diagnostic initial receive bound"));
        }
        let n = tokio::select! {_=stop.changed()=>return Ok(()),v=timeout(Duration::from_secs(profile.config.idle_timeout_seconds),socket.read(&mut buf[..cap]))=>v.map_err(|_|io::Error::other("initial read timeout"))??};
        if n == 0 {
            return rx.finish().map_err(io::Error::other);
        }
        rx.push(&buf[..n]).map_err(io::Error::other)?;
    }
}

pub async fn serve(profile: Validated, out: &Path) -> io::Result<()> {
    tokio::fs::create_dir_all(out).await?;
    // Bind the whole configured set before readiness; startup failure drops all.
    let mut bound = Vec::new();
    for &port in &profile.config.ports {
        bound.push(TcpListener::bind((profile.config.bind, port)).await?);
    }
    let file = tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(out.join("rust-events.jsonl"))
        .await?;
    let start = Instant::now();
    let boot = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(io::Error::other)?
        .as_nanos();
    let mut term = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    let (log, events) = mpsc::channel::<Value>(256);
    let mut writer = tokio::spawn(logging::write_events(file, events, Duration::from_secs(2)));
    let mut writer_result = None;
    let profile = Arc::new(profile);
    let (shutdown, stop) = watch::channel(false);
    let (accept_tx, mut accepted) = mpsc::channel::<(TcpStream, SocketAddr)>(32);
    let mut listeners = JoinSet::new();
    for listener in bound {
        let tx = accept_tx.clone();
        let mut stop = stop.clone();
        listeners.spawn(async move {
            loop {
                let pair=tokio::select! { _=stop.changed()=>return Ok::<_,io::Error>(()), p=listener.accept()=>p? };
                tokio::select! { _=stop.changed()=>return Ok(()), sent=tx.send(pair)=>if sent.is_err(){return Ok(());} }
            }
        });
    }
    drop(accept_tx);
    let quota = Arc::new(Semaphore::new(profile.config.max_connections));
    let mut workers = JoinSet::new();
    let mut next_id = 0u64;
    let mut failure = None;
    let outcome: io::Result<()> = async {
    emit(&log,json!({"event":"boot","pid":std::process::id(),"boot":boot.to_string(),"package":env!("CARGO_PKG_VERSION"),"source":option_env!("KR_SOURCE_REV").unwrap_or("unpublished"),"profile":if profile.bound.is_some(){"diagnostic_two_lobby"}else if profile.lobby.is_some(){"diagnostic_local_lobby"}else{"diagnostic_login_error_only"},"bootstrap_sha256":profile.config.bootstrap.sha256,"error_sha256":profile.config.login_error.sha256})).await?;
    state(out, true, false).await?;
    emit(
        &log,
        stamp(start, json!({"event":"ready","ports":profile.config.ports})),
    )
    .await?;
 loop {
        tokio::select! {
            result=&mut writer=>{
                writer_result=Some(logging::joined(result).and_then(|()|Err(io::Error::other("log writer stopped unexpectedly"))));
                break;
            },
            _=term.recv()=>break,
            value=tokio::signal::ctrl_c()=>{if let Err(e)=value {failure=Some(e);}break;},
            result=listeners.join_next(),if !listeners.is_empty()=>{failure=Some(io::Error::other(format!("listener stopped: {result:?}")));break;},
            result=workers.join_next(),if !workers.is_empty()=>if let Some(result)=result {match result {Ok(Ok(()))=>{},other=>{failure=Some(io::Error::other(format!("worker stopped: {other:?}")));break;}}},
            pair=accepted.recv()=>{
                let Some((socket,peer))=pair else {failure=Some(io::Error::other("all listeners stopped"));break;};
                let permitted=matches!(peer.ip(),IpAddr::V4(ip) if profile.config.allowed_peers.contains(&ip));
                let Ok(permit)=quota.clone().try_acquire_owned() else {drop(socket);emit(&log,stamp(start,json!({"event":"admission_rejected","reason":"connection_limit"}))).await?;continue;};
                if !permitted {drop(socket);drop(permit);emit(&log,stamp(start,json!({"event":"admission_rejected","reason":"peer_not_allowed"}))).await?;continue;}
                next_id+=1;let id=next_id;let local=socket.local_addr()?;
                emit(&log,stamp(start,json!({"event":"accepted","connection":id,"peer":peer.to_string(),"local":local.to_string(),"receive_sequence":40}))).await?;
                let profile=profile.clone();let log=log.clone();let stop=stop.clone();
                workers.spawn(async move {let _permit=permit;let result=connection(socket,id,profile,log.clone(),stop,start).await;emit(&log,stamp(start,json!({"event":"closed","connection":id,"reason":result.as_ref().err().map(ToString::to_string).unwrap_or_else(||"eof_or_shutdown".into())}))).await});
            }
        }
    }
    Ok(()) }.await;
    if let Err(error) = outcome {
        failure = Some(error);
    }
    let _ = shutdown.send(true);
    accepted.close();
    while let Ok(pair) = accepted.try_recv() {
        drop(pair);
    }
    if timeout(Duration::from_secs(3), async {
        while listeners.join_next().await.is_some() {}
        while workers.join_next().await.is_some() {}
    })
    .await
    .is_err()
    {
        listeners.abort_all();
        workers.abort_all();
        while listeners.join_next().await.is_some() {}
        while workers.join_next().await.is_some() {}
    }
    let state_result = state(out, false, true).await;
    let final_event = emit(&log,stamp(start,json!({"event":"stopped","connections":next_id,"error":failure.as_ref().map(ToString::to_string)}))).await;
    drop(log);
    logging::finish(writer, writer_result, final_event).await?;
    state_result?;
    if let Some(error) = failure {
        return Err(error);
    }
    Ok(())
}
