//! Bounded KR00967 diagnostic transport and opt-in two-profile account lobby adapter.
#![forbid(unsafe_code)]

pub mod account_bridge;
pub mod account_serving;
pub mod account_worker;
pub mod accounts;
pub mod binding;
mod bombs;
mod characters;
mod four_results;
mod life;
pub mod lobby;
mod logging;
pub mod network;
mod peer_routes;
mod pickups;
mod player_template;
pub mod room;
mod room_io;
mod room_packets;
pub mod room_registry;
mod room_relay;
mod room_service;
mod room_wire;
mod round_epochs;
mod rules;
mod terrain;
pub mod udp_relay;

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
    #[serde(default)]
    pub udp_relay: Option<udp_relay::Config>,
    pub schema: String,
    pub diagnostic_login_error_only: bool,
    pub bind: Ipv4Addr,
    pub allowed_peers: Vec<Ipv4Addr>,
    #[serde(default)]
    pub network_scope: network::NetworkScope,
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
    /// Explicit captured client metadata for controlled NAT measurements.
    /// Session/transport ownership still uses `peer`, never this field.
    #[serde(default)]
    pub reported_login_ipv4: Option<Ipv4Addr>,
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
    #[serde(default)]
    pub managed_rooms: Option<room_service::Config>,
    #[serde(default)]
    pub account_admission: Option<account_serving::Config>,
}
struct ActorProfile {
    identity: u32,
    name: Vec<u8>,
    packets: HashMap<String, Reply>,
    reported_login_ipv4: Option<Ipv4Addr>,
    configured_peer: Ipv4Addr,
}
struct BoundProfiles {
    managed: Option<Arc<room_service::Hub>>,
    actors: HashMap<room::PlayerId, ActorProfile>,
    legacy_peers: HashMap<Ipv4Addr, room::PlayerId>,
    registry: binding::Shared,
    rooms: Option<Arc<std::sync::Mutex<room_registry::Registry<room_io::Shared>>>>,
}
pub struct Validated {
    config: Config,
    bootstrap: Vec<u8>,
    error_header: [u8; HEADER_LEN],
    wcp_second: Vec<u8>,
    lobby: Option<HashMap<String, Reply>>,
    bound: Option<BoundProfiles>,
    accounts: Option<account_serving::Active>,
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
        || !config
            .network_scope
            .permits(config.bind, &config.allowed_peers, two)
        || config.ports.is_empty()
        || config.ports.len() > 32
        || config.ports.contains(&0)
        || config.ports.iter().copied().collect::<HashSet<_>>().len() != config.ports.len()
        || !network::admission_budget(config.max_connections, &config.allowed_peers)
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
        let ninth_probe = two
            .fixed_room
            .as_ref()
            .is_some_and(|room| room.ninth_admission_probe);
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
            || !(matches!(two.actors.len(), 2 | 4 | 8) || (ninth_probe && two.actors.len() == 9))
        {
            return Err(io::Error::other("invalid bounded diagnostic endpoints"));
        }
        if let Some(account) = &two.account_admission {
            if two.managed_rooms.is_none() && two.actors.len() != 2 {
                return Err(io::Error::other(
                    "account adapter supports exactly two measured profiles",
                ));
            }
            account.validate()?;
        }
        let mut actors = HashMap::new();
        let mut identities = HashSet::new();
        let mut legacy_peers = HashMap::new();
        for actor in &two.actors {
            let expected = match actor.identity {
                1 => "KRLocal",
                2 => "KRGuest",
                3 if matches!(two.actors.len(), 4 | 8 | 9) => "KRNew",
                4 if matches!(two.actors.len(), 4 | 8 | 9) => "KRPeer4",
                5 if matches!(two.actors.len(), 8 | 9) => "KRPeer5",
                6 if matches!(two.actors.len(), 8 | 9) => "KRPeer6",
                7 if matches!(two.actors.len(), 8 | 9) => "KRPeer7",
                8 if matches!(two.actors.len(), 8 | 9) => "KRPeer8",
                9 if ninth_probe && two.actors.len() == 9 => "KRPeer9",
                _ => return Err(io::Error::other("unsupported diagnostic identity")),
            };
            if actor.name != expected
                || !config.allowed_peers.contains(&actor.peer)
                || !identities.insert(actor.identity)
            {
                return Err(io::Error::other("invalid diagnostic actor binding"));
            }
            if let Some(reported) = actor.reported_login_ipv4
                && (config.network_scope != network::NetworkScope::ControlledInternet
                    || two.account_admission.is_none()
                    || reported.is_unspecified()
                    || reported.is_multicast()
                    || reported.is_broadcast())
            {
                return Err(io::Error::other(
                    "invalid controlled login metadata mapping",
                ));
            }
            let packets = load_lobby(&actor.profile, actor.identity, actor.name.as_bytes())?;
            if two.account_admission.is_some() {
                account_serving::validate_projection(&packets, actor.name.as_bytes())?;
            }
            if actors
                .insert(
                    actor.identity,
                    ActorProfile {
                        identity: actor.identity,
                        name: actor.name.as_bytes().to_vec(),
                        packets,
                        reported_login_ipv4: actor.reported_login_ipv4,
                        configured_peer: actor.peer,
                    },
                )
                .is_some()
            {
                return Err(io::Error::other("duplicate diagnostic endpoint"));
            }
        }
        for actor in &two.actors {
            if legacy_peers.insert(actor.peer, actor.identity).is_some()
                && two.account_admission.is_none()
            {
                return Err(io::Error::other("duplicate diagnostic endpoint"));
            }
        }
        // Every pinned actor must be allowed (checked above), but additional
        // allowed source IPs do not allocate or select a player identity.
        let room = two
            .fixed_room
            .as_ref()
            .map(|cfg| room_io::load(cfg, &actors))
            .transpose()?;
        let relay_requested = two
            .fixed_room
            .as_ref()
            .is_some_and(|r| r.endpoint_policy == rules::EndpointPolicy::Relay);
        if relay_requested != config.udp_relay.is_some()
            || (relay_requested && two.account_admission.is_none())
        {
            return Err(io::Error::other(
                "relay requires explicit room policy, network pool and account admission",
            ));
        }
        let managed = if let Some(cfg) = &two.managed_rooms {
            if two.account_admission.is_none() || ninth_probe {
                return Err(io::Error::other(
                    "managed rooms require account-owned profiles",
                ));
            }
            let prototype = room
                .as_ref()
                .ok_or_else(|| io::Error::other("managed rooms require room inputs"))?;
            let relay = config
                .udp_relay
                .as_ref()
                .map(room_relay::Factory::bind)
                .transpose()?;
            Some(room_service::Hub::new(cfg, prototype.clone(), relay)?)
        } else {
            None
        };
        let rooms = if managed.is_some() {
            None
        } else {
            room.map(|shared| -> io::Result<_> {
                let mut registry = room_registry::Registry::default();
                let key = registry.insert(1, room_registry::RoomId(1), shared.clone())?;
                if let Some(cfg) = &config.udp_relay {
                    let owner = room_relay::Owner::new(cfg, key)?;
                    room_io::install_relay(&shared, owner)?;
                }
                Ok(Arc::new(std::sync::Mutex::new(registry)))
            })
            .transpose()?
        };
        Some(BoundProfiles {
            managed,
            actors,
            legacy_peers,
            registry: binding::Shared::default(),
            rooms,
        })
    } else {
        if config.udp_relay.is_some() {
            return Err(io::Error::other("relay requires account room mode"));
        }
        None
    };
    Ok(Validated {
        config,
        bootstrap,
        error_header,
        wcp_second,
        lobby,
        bound,
        accounts: None,
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
            socket, id, &profile, packets, flow, None, None, None, log, stop, start,
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
    mut account: Option<account_serving::Lease>,
    initial: Option<(Receiver, kr_wire::Packet)>,
    log: Log,
    mut stop: watch::Receiver<bool>,
    start: Instant,
) -> io::Result<()> {
    let (mut rx, mut initial_packet) = match initial {
        Some((rx, packet)) => (rx, Some(packet)),
        None => (Receiver::new(40), None),
    };
    let mut account_opaque87 = 0usize;
    let mut next_auth_check = Instant::now() + Duration::from_secs(1);
    let mut managed_channel: Option<room_service::Channel> = None;
    let mut known_rooms = std::collections::BTreeSet::new();
    let mut catalog_dirty = false;
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
            // Native account-lobby trace: exact body matches typed u16 zero.
            // Semantics unknown; bounded observation only, no ACK/activity/state effect.
            if matches!(account, Some(account_serving::Lease::Channel(_)))
                && flow.stage == lobby::Stage::Lobby
                && room_session.as_ref().is_none_or(room_io::Session::in_lobby)
                && packet.opcode == 87
                && packet.payload
                    == kr_wire::write_uint(0, kr_wire::Width::U16).map_err(io::Error::other)?
            {
                account_opaque87 += 1;
                if account_opaque87 > 16 {
                    return Err(io::Error::other("account opaque87 budget"));
                }
                emit(&log,stamp(start,json!({"event":"account_opaque87_observed","connection":id,"sequence":packet.sequence,"payload_bytes":packet.payload.len(),"policy":"exact_native_profile_no_ack_no_state_effect"}))).await?;
                continue;
            }
            if packet.opcode == 67
                && let Some(account) = &mut account
            {
                tokio::select! { _=stop.changed()=>return Ok(()), value=account.authorize(true)=>value? };
            }
            if let Some(channel) = &managed_channel
                && matches!(packet.opcode, 81 | 82)
            {
                if room_session.is_some() {
                    return Err(io::Error::other("room selection before completed return"));
                }
                let Some(account_serving::Lease::Channel(lease)) = &mut account else {
                    return Err(io::Error::other("managed room has no account owner"));
                };
                tokio::select! { _=stop.changed()=>return Ok(()), value=lease.check()=>value.map_err(|e|io::Error::other(format!("room selection denied: {e:?}")))? };
                let peer = match socket.peer_addr()?.ip() {
                    IpAddr::V4(peer) => peer,
                    _ => return Err(io::Error::other("account IPv4 required")),
                };
                let selected = channel.select(
                    packet.opcode,
                    &packet.payload,
                    peer,
                    lease.relay_authority(),
                )?;
                match selected {
                    room_service::Selection::Joined(session) => room_session = Some(session),
                    room_service::Selection::Full => {
                        // CP227/252 generic87/result2. Capacity is our policy
                        // reason; this is not an original full-room error code.
                        let body =
                            kr_wire::write_uint(2, kr_wire::Width::U8).map_err(io::Error::other)?;
                        let frame = kr_wire::kr00967::encode(87, &body, send_sequence)
                            .map_err(io::Error::other)?;
                        timeout(
                            Duration::from_millis(profile.config.write_timeout_ms),
                            socket.write_all(&frame),
                        )
                        .await
                        .map_err(|_| io::Error::other("full-room reply timeout"))??;
                        emit(&log, stamp(start, json!({"event":"managed_join_rejected","connection":id,
                            "identity":flow.identity(),"reason":"room_full","opcode":87,
                            "sequence":send_sequence,"bytes":frame.len(),"sha256":sha256_hex(&frame)}))).await?;
                        send_sequence = send_sequence.wrapping_add(3);
                        continue;
                    }
                }
                pending.clear();
                emit(&log,stamp(start,json!({"event":"managed_room_selection","connection":id,"identity":flow.identity(),"opcode":packet.opcode}))).await?;
                continue;
            }
            if let Some(session) = &room_session
                && room_io::Session::handles(packet.opcode)
            {
                if let Some(account) = &mut account {
                    // Current room commands are activity. Literal opaque79 is
                    // observation-only and must not extend account idle expiry.
                    // No lock crosses DB await; Session rechecks membership.
                    tokio::select! { _=stop.changed()=>return Ok(()), value=account.authorize(packet.opcode != 79)=>value? };
                }
                let observation = session.packet(packet.opcode, &packet.payload)?;
                emit(&log,stamp(start,json!({"event":"room_request","connection":id,"identity":flow.identity(),"opcode":packet.opcode,"sequence":packet.sequence,"payload_bytes":packet.payload.len(),"dropped_after_abort":observation.dropped_after_abort,"start_rejection":observation.start_rejection,"admission_rejection":observation.admission_rejection,"pickup":observation.pickup,"bomb":observation.bomb,"terrain_notice":observation.terrain_notice,"life":observation.life}))).await?;
                continue;
            }
            let aliases = match flow.packet(packet.opcode, &packet.payload) {
                Ok(aliases) => aliases,
                Err(error) => {
                    emit(&log,stamp(start,json!({"event":"lobby_request_denied","connection":id,"opcode":packet.opcode,"sequence":packet.sequence,"payload_bytes":packet.payload.len(),"stage":format!("{:?}",flow.stage)}))).await?;
                    return Err(error);
                }
            };
            if packet.opcode == 6
                && let Some(lease) = &lease
            {
                lease.bind_channel()?;
            }
            emit(&log,stamp(start,json!({"event":"local_lobby_request","connection":id,"opcode":packet.opcode,"sequence":packet.sequence,"payload_bytes":packet.payload.len(),"stage":format!("{:?}",flow.stage)}))).await?;
            if packet.opcode == 67
                && flow.stage == lobby::Stage::Lobby
                && let Some(hub) = profile.bound.as_ref().and_then(|b| b.managed.as_ref())
            {
                managed_channel = Some(hub.register(flow.identity(), id)?);
            }
            for alias in aliases {
                if alias == "roomlist_A"
                    && let Some(channel) = &mut managed_channel
                {
                    send_catalog(
                        &mut socket,
                        channel,
                        &mut known_rooms,
                        &mut send_sequence,
                        profile,
                        &log,
                        id,
                        start,
                    )
                    .await?;
                    continue;
                }
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
                if alias == "redirect"
                    && let Some(account) = &mut account
                {
                    account.redirected()?;
                }
                send_sequence = send_sequence.wrapping_add(3);
            }
            if packet.opcode == 67
                && flow.stage == lobby::Stage::Lobby
                && let Some(registry) = profile.bound.as_ref().and_then(|b| b.rooms.as_ref())
            {
                // Keep registry selection and channel admission synchronous under
                // registry -> room lock order; no await while either lock is held.
                let registry = registry
                    .lock()
                    .map_err(|_| io::Error::other("room registry unavailable"))?;
                let key = registry.selection(1, room_registry::RoomId(1))?;
                let shared = registry.get(key)?;
                room_session = Some(if account.is_some() {
                    let peer = match socket.peer_addr()?.ip() {
                        IpAddr::V4(peer) => peer,
                        _ => return Err(io::Error::other("account endpoint requires IPv4")),
                    };
                    let authority = match &account {
                        Some(account_serving::Lease::Channel(channel)) => {
                            Some(channel.relay_authority())
                        }
                        _ => None,
                    };
                    room_io::Session::open_with_authority(
                        shared,
                        flow.identity(),
                        id,
                        Some(peer),
                        authority,
                    )?
                } else {
                    room_io::Session::open(shared, flow.identity(), id)?
                });
            }
        }
        if account.is_some() && Instant::now() >= next_auth_check {
            if let Some(account) = &mut account {
                tokio::select! { _=stop.changed()=>return Ok(()), value=account.authorize(false)=>value? };
            }
            next_auth_check = Instant::now() + Duration::from_secs(1);
        }
        if let Some(session) = &room_session {
            if let Some(account) = &mut account {
                tokio::select! { _=stop.changed()=>return Ok(()), value=account.authorize(false)=>value? };
            }
            session.tick()?;
            let stats =
                tokio::select! {_=stop.changed()=>return Ok(()),value=session.pump_relay()=>value?};
            if stats.received > 0 || stats.sent > 0 {
                emit(&log,stamp(start,json!({"event":"relay_pump","connection":id,
                    "received":stats.received,"accepted":stats.accepted,"dropped":stats.dropped,"sent":stats.sent,
                    "policy":"opt_in_candidate_not_native_evidence"}))).await?;
            }
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
            if let Some(account) = &mut account {
                tokio::select! { _=stop.changed()=>return Ok(()), value=account.authorize(false)=>value? };
            }
            if !session.valid(&batch) {
                emit(&log,stamp(start,json!({"event":"room_queued_dropped","connection":id,"reason":"membership_phase_generation"}))).await?;
                continue;
            }
            for (alias, reply) in &batch.packets {
                if !session.valid(&batch) {
                    return Err(io::Error::other("room membership changed during batch"));
                }
                let Some(mut reply) = session.project(reply)? else {
                    continue;
                };
                if reply.opcode == 91 && managed_channel.is_some() {
                    continue;
                }
                if reply.opcode == 91 {
                    let rooms = profile
                        .bound
                        .as_ref()
                        .and_then(|bound| bound.rooms.as_ref())
                        .ok_or_else(|| io::Error::other("missing room registry"))?;
                    // Build the list from current room entries under synchronous
                    // registry -> room lock order; release before socket await.
                    reply.payload = {
                        let registry = rooms
                            .lock()
                            .map_err(|_| io::Error::other("room registry unavailable"))?;
                        registry.listing91()?
                    };
                }
                let frame = kr_wire::kr00967::encode(reply.opcode, &reply.payload, send_sequence)
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
            if let Some(channel) = &managed_channel
                && batch.packets.iter().any(|(alias, _)| alias == "return241")
            {
                room_session.take();
                pending.clear();
                channel.returned()?;
                catalog_dirty = true;
            }
        }
        if catalog_dirty
            && room_session.is_none()
            && let Some(channel) = &mut managed_channel
        {
            if let Some(account) = &mut account {
                tokio::select! { _=stop.changed()=>return Ok(()), value=account.authorize(false)=>value? };
            }
            send_catalog(
                &mut socket,
                channel,
                &mut known_rooms,
                &mut send_sequence,
                profile,
                &log,
                id,
                start,
            )
            .await?;
            catalog_dirty = false;
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
            change=async { match &mut managed_channel { Some(channel)=>channel.changes.changed().await, None=>std::future::pending().await }}=>{
                change.map_err(|_| io::Error::other("room catalog closed"))?;
                catalog_dirty = true; continue;
            },
            _=tokio::time::sleep_until(next_auth_check.into()),if account.is_some()=>continue,
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
async fn send_catalog(
    socket: &mut TcpStream,
    channel: &mut room_service::Channel,
    known: &mut std::collections::BTreeSet<(u16, u32)>,
    sequence: &mut u32,
    profile: &Validated,
    log: &Log,
    id: u64,
    start: Instant,
) -> io::Result<()> {
    let catalog = channel.catalog(known)?;
    for (opcode, payload) in catalog.packets {
        let frame =
            kr_wire::kr00967::encode(opcode, &payload, *sequence).map_err(io::Error::other)?;
        timeout(
            Duration::from_millis(profile.config.write_timeout_ms),
            socket.write_all(&frame),
        )
        .await
        .map_err(|_| io::Error::other("catalog reply timeout"))??;
        emit(
            log,
            stamp(
                start,
                json!({"event":"managed_catalog_reply","connection":id,"opcode":opcode,
            "sequence":*sequence,"bytes":frame.len(),"sha256":sha256_hex(&frame)}),
            ),
        )
        .await?;
        *sequence = sequence.wrapping_add(3);
    }
    *known = catalog.tuples;
    Ok(())
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
    if let Some(accounts) = &profile.accounts {
        return account_serving::connection(socket, id, profile, accounts, log, stop, start).await;
    }
    let bound = profile.bound.as_ref().expect("validated bound mode");
    let peer = match socket.peer_addr()?.ip() {
        IpAddr::V4(ip) => ip,
        _ => return Err(io::Error::other("IPv4 diagnostic only")),
    };
    let actor = bound
        .legacy_peers
        .get(&peer)
        .and_then(|identity| bound.actors.get(identity))
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
                None,
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

pub async fn serve(mut profile: Validated, out: &Path) -> io::Result<()> {
    tokio::fs::create_dir_all(out).await?;
    let mut account_owner = if let Some(config) = profile
        .config
        .diagnostic_two_lobby
        .as_ref()
        .and_then(|c| c.account_admission.as_ref())
    {
        let profiles = profile
            .bound
            .as_ref()
            .expect("validated account profiles")
            .actors
            .values()
            .map(|a| (a.identity, a.name.clone()))
            .collect();
        Some(account_serving::Owner::open(config, profiles).await?)
    } else {
        None
    };
    profile.accounts = account_owner.as_ref().map(|a| a.active.clone());
    let startup: io::Result<_> = async {
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
        let term = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
        Ok((bound, file, start, boot, term))
    }
    .await;
    let (bound, file, start, boot, mut term) = match startup {
        Ok(resources) => resources,
        Err(error) => {
            if let Some(owner) = account_owner.take()
                && let Err(cleanup) = owner.shutdown().await
            {
                return Err(io::Error::other(format!(
                    "startup: {error}; account cleanup: {cleanup}"
                )));
            }
            return Err(error);
        }
    };
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
    if let Some(owner) = &mut account_owner {
        let listener = owner.listener.take().expect("owned launch listener");
        listeners.spawn(account_serving::control(
            listener,
            owner.active.clone(),
            profile.config.allowed_peers.clone(),
            stop.clone(),
        ));
    }
    let mut account_health = tokio::time::interval(Duration::from_secs(1));
    let quota = Arc::new(Semaphore::new(profile.config.max_connections));
    let mut workers = JoinSet::new();
    let mut next_id = 0u64;
    let mut failure = None;
    let outcome: io::Result<()> = async {
    emit(&log,json!({"event":"boot","pid":std::process::id(),"boot":boot.to_string(),"package":env!("CARGO_PKG_VERSION"),"source":option_env!("KR_SOURCE_REV").unwrap_or("unpublished"),"profile":if profile.accounts.is_some(){"account_two_profile_lobby"}else if profile.bound.is_some(){"diagnostic_two_lobby"}else if profile.lobby.is_some(){"diagnostic_local_lobby"}else{"diagnostic_login_error_only"},"bootstrap_sha256":profile.config.bootstrap.sha256,"error_sha256":profile.config.login_error.sha256})).await?;
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
            _=account_health.tick(),if profile.accounts.is_some()=>{
                if profile.accounts.as_ref().is_some_and(|a|a.client.is_closed()) {
                    failure=Some(io::Error::other("account worker unavailable"));break;
                }
            },
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
    if let Some(owner) = account_owner
        && let Err(error) = owner.shutdown().await
    {
        failure.get_or_insert(error);
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
