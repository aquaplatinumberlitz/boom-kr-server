//! Fixed-room adapters with bounded recipient queues and membership fences.
use super::{ActorProfile, LobbyPacketPin, Reply};
use crate::bombs;
use crate::room_packets::{Template, normalize_legacy, packet};
use crate::{
    pickups,
    room::{Member, Membership, Phase, PlayerId, Room},
    room_wire,
    rules::{self, DeparturePolicy, ResultPolicy, RoomRules, TeamPolicy},
};
use kr_wire::kr00967 as wire;
use kr_wire::{Width, decode, read_uint};
use serde::Deserialize;
use std::{
    collections::{HashMap, HashSet},
    io,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::sync::mpsc;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default)]
    pub(crate) endpoint_policy: rules::EndpointPolicy,
    pub title: String,
    pub packets: HashMap<String, LobbyPacketPin>,
    #[serde(default)]
    pub(crate) slot_allocation: rules::SlotAllocation,
    #[serde(default)]
    pub terrain: Option<crate::terrain::Config>,
    #[serde(default)]
    pub battle: Option<BattleConfig>,
    /// Opt-in measured CP201 capacity1/2/1/1, power1, design fuse6 seconds.
    #[serde(default)]
    pub four_owner_bombs: bool,
    #[serde(default)]
    pub four_life_reports: bool,
    #[serde(default)]
    pub four_results: bool,
    #[serde(default)]
    pub four_abort_room: bool,
    /// CP208–210 explicit fixed8 baseline; not four abort/items/terrain.
    #[serde(default)]
    pub eight_battle: bool,
    /// Explicit CP213 fixed operator teams, only with eight_battle.
    #[serde(default)]
    pub eight_team_layout: Option<crate::room::EightTeamLayout>,
    /// Opt-in ninth diagnostic lobby actor for the full-room negative consumer.
    #[serde(default)]
    pub ninth_admission_probe: bool,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BattleConfig {
    pub mode: String,
    pub packets: HashMap<String, LobbyPacketPin>,
}
/// Operator-selected measured roster profile; no character/team UI protocol.
#[derive(Clone, Copy, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ManagedRoundMode {
    #[default]
    Duel,
    Teams,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManagedRoundConfig {
    #[serde(default)]
    pub mode: ManagedRoundMode,
    #[serde(default = "duel_teams")]
    pub teams: Vec<u8>,
    #[serde(default)]
    pub battle: Option<BattleConfig>,
    pub initial_bomb_capacity: Vec<usize>,
}
fn duel_teams() -> Vec<u8> {
    vec![0, 1]
}
pub(crate) struct ManagedRound {
    bodies: HashMap<Template, Vec<u8>>,
    initial_bomb_capacity: Vec<usize>,
    rules: RoomRules,
    limits: rules::ReportLimits,
}
pub(crate) fn load_managed_round(
    cfg: &ManagedRoundConfig,
    prototype: &Shared,
) -> io::Result<Arc<ManagedRound>> {
    let (start, rules, limits) = {
        let source = prototype
            .lock()
            .map_err(|_| err("room prototype unavailable"))?;
        if source.rules.endpoint != rules::EndpointPolicy::LegacyDirect
            || source.model.terrain.is_some()
        {
            return Err(err(
                "managed rounds require direct UDP and no terrain profile",
            ));
        }
        let count = cfg.teams.len();
        if count > source.model.capacity()
            || cfg.teams != source.model.fixed_teams()[..count]
            || cfg.initial_bomb_capacity != rules::default_bomb_capacity(count)
        {
            return Err(err("unsupported managed roster/team/slot limits"));
        }
        let (result, departure, bomb, life, teams) = match cfg.mode {
            ManagedRoundMode::Duel if count == 2 && cfg.battle.is_some() => (
                ResultPolicy::Duel,
                DeparturePolicy::Forfeit,
                rules::BombPolicy::Basic,
                rules::LifePolicy::DuelReports,
                TeamPolicy::Any,
            ),
            ManagedRoundMode::Teams if matches!(count, 4 | 8) && cfg.battle.is_none() => (
                ResultPolicy::Teams,
                DeparturePolicy::FenceActive,
                rules::BombPolicy::OwnerReports,
                rules::LifePolicy::TeamReports,
                TeamPolicy::BalancedManner,
            ),
            _ => {
                return Err(err(
                    "unsupported managed round mode/count/battle combination",
                ));
            }
        };
        let rules = RoomRules {
            roster: rules::RoundRoster::Contiguous(count),
            result,
            departure,
            bomb,
            life,
            teams,
            slot_allocation: rules::SlotAllocation::FirstVacant,
            endpoint: rules::EndpointPolicy::LegacyDirect,
            ninth_admission_probe: false,
        };
        (
            wire::start121_roster(source.body(Template::Start121), &cfg.teams)?,
            rules,
            rules::ReportLimits::measured(count)?,
        )
    }; // All disk reads/fixture decode happen after releasing the prototype lock.
    let mut bodies = HashMap::from([(Template::Start121, start)]);
    if let Some(battle) = &cfg.battle {
        if battle.mode != "post_blast_untrapped_drained" || battle.packets.len() != 4 {
            return Err(err("unsupported managed Battle profile"));
        }
        for (alias, key, size) in [
            ("rank", Template::SlotOneWins175, 302),
            ("rank_Awin", Template::SlotZeroWins175, 302),
            ("rank_all_dead", Template::AllDead175, 302),
            ("return", Template::Return180, 3),
        ] {
            let pin = battle
                .packets
                .get(alias)
                .ok_or_else(|| err("missing managed Battle fixture"))?;
            let raw = pin.fixture.load()?;
            let decoded = decode(
                &raw,
                pin.sequence
                    .ok_or_else(|| err("missing managed fixture sequence"))?,
            )
            .map_err(io::Error::other)?;
            if pin.opcode != key.opcode()
                || decoded.opcode != key.opcode()
                || decoded.payload.len() != size
                || !wire::header_matches(&raw[..37])
            {
                return Err(err(
                    "unsupported managed Battle fixture opcode/shape/header",
                ));
            }
            if key == Template::Return180 {
                wire::selector_reply(&decoded.payload, 180, 0)?;
            }
            bodies.insert(key, decoded.payload);
        }
    }
    Ok(Arc::new(ManagedRound {
        bodies,
        initial_bomb_capacity: cfg.initial_bomb_capacity.clone(),
        rules,
        limits,
    }))
}

pub type Shared = Arc<Mutex<Runtime>>;
pub(crate) fn install_relay(shared: &Shared, owner: crate::room_relay::Owner) -> io::Result<()> {
    let mut runtime = shared.lock().map_err(|_| err("room unavailable"))?;
    if runtime.rules.endpoint != rules::EndpointPolicy::Relay
        || !runtime.channels.is_empty()
        || runtime.relay.is_some()
    {
        return Err(err("relay install requires an empty opt-in room"));
    }
    runtime.relay = Some(owner);
    Ok(())
}
struct Channel {
    connection: u64,
    sender: mpsc::Sender<Batch>,
    endpoint: u32,
    relay_authority: Option<crate::account_bridge::RelayAuthority>,
}
#[derive(Clone)]
pub enum Guard {
    Channel,
    Bomb {
        member: Membership,
        ticket: bombs::Ticket,
    },
    Member(Membership, Option<u64>),
    Outcome(Membership, u64),
    Life {
        recipient: Membership,
        source: Membership,
        round: u64,
        clear21: Option<(String, Instant)>,
    },
    Lobby {
        identity: u32,
        connection: u64,
        visit: u64,
    },
}
pub struct Batch {
    origin: Option<(Arc<()>, u32, u64)>,
    pub guard: Guard,
    pub due: Instant,
    pub packets: Vec<(String, Reply)>,
    pub setup: Option<Membership>,
    pub subject: Option<Membership>,
    pub bomb_complete: Option<(bombs::Ticket, Membership)>,
    pub life_complete: Option<(Membership, u64, String, Membership)>,
    pub outcome_complete: Option<(Membership, u64)>,
    pub abort_return_complete: Option<(Membership, u64)>,
}
pub struct Runtime {
    public_key: Option<crate::room_registry::RoomKey>,
    managed_round: Option<Arc<ManagedRound>>,
    relay: Option<crate::room_relay::Owner>,
    scope: Arc<()>,
    model: Room,
    players: HashMap<PlayerId, Reply>,
    bodies: HashMap<Template, Vec<u8>>,
    channels: HashMap<u32, Channel>,
    last_visits: HashMap<u32, u64>,
    requested: HashSet<Membership>,
    rostered: HashSet<(Membership, Membership)>,
    title: Vec<u8>,
    selector: u8,
    ready: Reply,
    ips: HashMap<u32, u32>,
    ports: HashMap<Membership, u16>,
    floor_catalog: pickups::Catalog,
    rules: RoomRules,
    abort: Option<FourAbort>,
}
/// Single-loss CP205 diagnostic lifecycle; preserved through final cleanup.
/// A second loss during recovery fails closed, rather than claiming recovery.
type AbortTransition = (Room, FourAbort, Vec<(u32, Batch)>);

struct FourAbort {
    round: u64,
    departed: Member,
    returned: HashSet<Membership>,
    setups: HashSet<Membership>,
    refresh_queued: bool,
    completed: bool,
}
#[derive(serde::Serialize)]
pub struct BombObservation {
    pub accepted: bool,
    pub id: Option<u16>,
    pub reason: &'static str,
}
#[derive(Default)]
pub struct Observation {
    pub dropped_after_abort: bool,
    pub start_rejection: Option<&'static str>,
    pub admission_rejection: Option<&'static str>,
    pub pickup: Option<pickups::Decision>,
    pub bomb: Option<BombObservation>,
    pub terrain_notice: Option<&'static str>,
    pub life: Option<crate::life::Decision>,
}
pub struct Session {
    shared: Shared,
    pub inbox: mpsc::Receiver<Batch>,
    identity: u32,
    connection: u64,
}
fn err(s: &str) -> io::Error {
    io::Error::other(s.to_owned())
}
fn legacy_player_alias(id: u32) -> io::Result<&'static str> {
    match id {
        1 => Ok("player_A"),
        2 => Ok("player_B"),
        3 => Ok("player_C"),
        4 => Ok("player_D"),
        5 => Ok("player_E"),
        6 => Ok("player_F"),
        7 => Ok("player_G"),
        8 => Ok("player_H"),
        _ => Err(err("unsupported diagnostic player identity")),
    }
}
pub fn load(cfg: &Config, actors: &HashMap<PlayerId, ActorProfile>) -> io::Result<Shared> {
    if cfg.title.is_empty()
        || cfg.title.len() > 32
        || !cfg.title.bytes().all(|b| (32..=126).contains(&b))
    {
        return Err(err("fixed room title must be bounded ASCII"));
    }
    let actor_count = actors.len();
    let (rules, count) = RoomRules::from_config(cfg, actor_count)?;
    if !matches!(count, 2 | 4 | 8)
        || actors
            .values()
            .any(|a| a.identity == 0 || a.identity as usize > actor_count)
    {
        return Err(err("unsupported diagnostic roster"));
    }
    // Four profile first covers startup/room/loading. Reject unported terrain and
    // result policies before binding rather than silently selecting two-player ranks.
    if matches!(count, 4 | 8) && (cfg.battle.is_some() || cfg.terrain.is_some()) {
        return Err(err("four-player Battle/terrain policy not ported"));
    }
    let mut expected = vec![
        ("create", 86, 11),
        ("join", 87, 7),
        ("listing", 91, 0),
        ("table", 238, 72),
        ("map", 117, 8),
        ("floor", 582, 46),
        ("start", 121, 129 + 26 * (count - 2)),
        ("go", 197, 14),
        ("play", 196, 7),
        ("peer", 153, 61 + 18 * count),
    ];
    for actor in actors.values() {
        if cfg.ninth_admission_probe && actor.identity == 9 {
            continue;
        }
        expected.push((
            legacy_player_alias(actor.identity)?,
            106,
            190 + 2 * actor.name.len(),
        ));
    }
    if cfg.packets.len() != expected.len() {
        return Err(err("fixed room packet set mismatch"));
    }
    let mut packets = HashMap::new();
    for (alias, opcode, size) in expected {
        let pin = cfg
            .packets
            .get(alias)
            .ok_or_else(|| err("missing room packet"))?;
        let raw = pin.fixture.load()?;
        let packet = decode(
            &raw,
            pin.sequence
                .ok_or_else(|| err("room requires captured frame sequence"))?,
        )
        .map_err(io::Error::other)?;
        if pin.opcode != opcode
            || packet.opcode != opcode
            || (size != 0 && packet.payload.len() != size)
        {
            return Err(err("room opcode/size mismatch"));
        }
        if opcode != 106 && !wire::header_matches(&raw[..37]) {
            return Err(err("unsupported room header profile"));
        }
        packets.insert(
            alias.to_owned(),
            Reply {
                header: raw[..37].try_into().expect("decoded header"),
                opcode,
                payload: packet.payload,
            },
        );
    }
    if packets["listing"].payload.len() < 16
        || read_uint(&packets["listing"].payload[..2], Width::U16).map_err(io::Error::other)? != 1
    {
        return Err(err("expected measured single room listing"));
    }
    if wire::listing91_title(&packets["listing"].payload)? != cfg.title.as_bytes() {
        return Err(err("fixed listing title mismatch"));
    }
    // Normalize the old per-player fixture interface once. Runtime projections
    // come from one template; a changed opaque field must fail before bind.
    let template_reply = packets[legacy_player_alias(1)?].clone();
    let template = crate::player_template::PlayerTemplate::parse(&template_reply.payload)?;
    if !wire::header_matches(&template_reply.header) {
        return Err(err("unsupported room header profile"));
    }
    let mut players = HashMap::new();
    for actor in actors.values() {
        if cfg.ninth_admission_probe && actor.identity == 9 {
            continue;
        }
        let captured = &packets[legacy_player_alias(actor.identity)?];
        let mut body = template.project(actor.identity, &actor.name)?;
        let slot = (actor.identity - 1) as u8;
        body = wire::player106_state(&body, slot, slot % 2, false, actor.identity == 1)?;
        let mut template_header = template_reply.header;
        let mut captured_header = captured.header;
        // The codec recomputes length for the shorter measured name5 profile.
        template_header[13..17].fill(0);
        captured_header[13..17].fill(0);
        if body != captured.payload || template_header != captured_header {
            return Err(err("room player common-template mismatch"));
        }
        players.insert(
            actor.identity,
            Reply {
                header: template_reply.header,
                opcode: 106,
                payload: body,
            },
        );
    }
    let mut ready = actors
        .values()
        .next()
        .ok_or_else(|| err("missing actors"))?
        .packets["ready"]
        .clone();
    if !wire::header_matches(&ready.header) {
        return Err(err("unsupported room header profile"));
    }
    ready.header = wire::HEADER;
    let packets_floor_body = packets["floor"].payload.clone();
    let terrain = cfg
        .terrain
        .as_ref()
        .map(|spec| {
            spec.validate(
                &cfg.packets["map"].fixture.load()?,
                &cfg.packets["start"].fixture.load()?,
            )
        })
        .transpose()?;
    if let Some(battle) = &cfg.battle {
        if battle.mode != "post_blast_untrapped_drained" || battle.packets.len() != 4 {
            return Err(err("unsupported Battle profile"));
        }
        for (alias, op, size) in [
            ("rank", 175, 302),
            ("rank_Awin", 175, 302),
            ("rank_all_dead", 175, 302),
            ("return", 180, 3),
        ] {
            let pin = battle
                .packets
                .get(alias)
                .ok_or_else(|| err("missing Battle fixture"))?;
            let raw = pin.fixture.load()?;
            let packet = decode(
                &raw,
                pin.sequence
                    .ok_or_else(|| err("missing Battle fixture sequence"))?,
            )
            .map_err(io::Error::other)?;
            if pin.opcode != op || packet.opcode != op || packet.payload.len() != size {
                return Err(err("unsupported Battle fixture opcode/size"));
            }
            if !wire::header_matches(&raw[..37]) {
                return Err(err("unsupported room header profile"));
            }
            if op == 180 {
                wire::selector_reply(&packet.payload, 180, 0)?;
            }
            packets.insert(
                alias.into(),
                Reply {
                    header: raw[..37].try_into().expect("decoded header"),
                    opcode: op,
                    payload: packet.payload,
                },
            );
        }
    }
    let mut model = Room::with_allocation(count, rules.slot_allocation)?;
    if let Some(layout) = cfg.eight_team_layout {
        model.set_eight_layout(layout)?;
    }
    if cfg.eight_battle {
        let start = packets.get_mut("start").expect("validated start");
        start.payload = wire::start121_teams(&start.payload, model.fixed_teams())?;
    }
    model.terrain = terrain;
    Ok(Arc::new(Mutex::new(Runtime {
        public_key: None,
        managed_round: None,
        relay: None,
        scope: Arc::new(()),
        model,
        players,
        bodies: normalize_legacy(&packets),
        channels: HashMap::new(),
        last_visits: HashMap::new(),
        requested: HashSet::new(),
        rostered: HashSet::new(),
        title: cfg.title.as_bytes().to_vec(),
        selector: 0,
        ready,
        ips: actors
            .values()
            .map(|a| (a.identity, u32::from(a.configured_peer)))
            .collect(),
        ports: HashMap::new(),
        floor_catalog: pickups::Catalog::from_floor(&packets_floor_body)?,
        rules,
        abort: None,
    })))
}
/// Clone immutable, prevalidated inputs only. The prototype never serves a channel.
/// Managed selection currently exposes room setup, not dynamic Battle acceptance.
pub(crate) fn fork(
    prototype: &Shared,
    key: crate::room_registry::RoomKey,
    epochs: Arc<crate::round_epochs::RoundEpochs>,
    managed_round: Option<Arc<ManagedRound>>,
) -> io::Result<Shared> {
    let source = prototype
        .lock()
        .map_err(|_| err("room prototype unavailable"))?;
    if !source.channels.is_empty() || !source.model.members.is_empty() || source.relay.is_some() {
        return Err(err("room prototype is not pristine"));
    }
    let mut model = source.model.clone();
    model.set_allocation(rules::SlotAllocation::FirstVacant)?;
    model.set_round_epochs(epochs)?;
    let mut rules = source.rules;
    rules.slot_allocation = rules::SlotAllocation::FirstVacant;
    if let Some(profile) = &managed_round {
        rules = profile.rules;
        model.round_roster = rules.roster;
    }

    Ok(Arc::new(Mutex::new(Runtime {
        public_key: Some(key),
        managed_round,
        relay: None,
        scope: Arc::new(()),
        model,
        players: source.players.clone(),
        bodies: source.bodies.clone(),
        channels: HashMap::new(),
        last_visits: HashMap::new(),
        requested: HashSet::new(),
        rostered: HashSet::new(),
        title: source.title.clone(),
        selector: source.selector,
        ready: source.ready.clone(),
        ips: source.ips.clone(),
        ports: HashMap::new(),
        floor_catalog: source.floor_catalog.clone(),
        rules,
        abort: None,
    })))
}
pub(crate) fn detach(shared: &Shared, identity: u32, connection: u64) -> io::Result<()> {
    shared
        .lock()
        .map_err(|_| err("room unavailable"))?
        .detach(identity, connection);
    Ok(())
}
pub(crate) fn removable(shared: &Shared) -> bool {
    shared.lock().is_ok_and(|r| {
        r.channels.is_empty()
            && r.model.members.is_empty()
            && r.model.phase == Phase::Idle
            && r.abort.is_none()
    })
}
pub(crate) fn listing_row(shared: &Shared) -> io::Result<wire::ListingRow> {
    let runtime = shared.lock().map_err(|_| err("room unavailable"))?;
    let mut rows = wire::listing91_parse_rows(runtime.body(Template::Listing91))?;
    if rows.len() != 1 {
        return Err(err("expected one room template row"));
    }
    let mut row = rows.remove(0);
    row.title = runtime.title.clone();
    if runtime.public_key.is_some() {
        row.set_population(runtime.model.members.len(), runtime.model.capacity())?;
    }
    Ok(row)
}
impl Runtime {
    fn member(&self, id: u32, connection: u64) -> io::Result<Member> {
        self.model
            .members
            .get(&id)
            .filter(|m| m.key.connection == connection)
            .cloned()
            .ok_or_else(|| err("missing/current room membership"))
    }
    fn body(&self, key: Template) -> &[u8] {
        self.managed_round
            .as_ref()
            .and_then(|profile| profile.bodies.get(&key))
            .unwrap_or_else(|| &self.bodies[&key])
    }
    fn reply(&self, key: Template, body: Option<Vec<u8>>) -> Reply {
        packet(
            key.opcode(),
            body.unwrap_or_else(|| self.body(key).to_vec()),
        )
    }
    fn player(&self, m: &Member, host: Option<PlayerId>) -> io::Result<Reply> {
        let reply = self
            .players
            .get(&m.key.identity)
            .ok_or_else(|| err("missing current player projection"))?
            .clone();
        Ok(packet(
            106,
            room_wire::player(&reply.payload, m, host == Some(m.key.identity))?,
        ))
    }
    fn slot(&self, m: &Member, host: Option<PlayerId>) -> io::Result<Reply> {
        let mut reply = self.player(m, host)?;
        reply.opcode = 110;
        reply.payload = room_wire::slot(&reply.payload, m, host == Some(m.key.identity))?;
        Ok(reply)
    }
    fn member_batch(
        key: Membership,
        round: Option<u64>,
        delay: u64,
        packets: Vec<(String, Reply)>,
    ) -> Batch {
        Batch {
            origin: None,
            guard: Guard::Member(key, round),
            due: Instant::now() + Duration::from_millis(delay),
            packets,
            setup: None,
            subject: None,
            bomb_complete: None,
            life_complete: None,
            outcome_complete: None,
            abort_return_complete: None,
        }
    }
    fn lobby_batch(
        &self,
        id: u32,
        connection: u64,
        delay: u64,
        packets: Vec<(String, Reply)>,
    ) -> Batch {
        Batch {
            origin: None,
            guard: Guard::Lobby {
                identity: id,
                connection,
                visit: *self.last_visits.get(&id).unwrap_or(&0),
            },
            due: Instant::now() + Duration::from_millis(delay),
            packets,
            setup: None,
            subject: None,
            bomb_complete: None,
            life_complete: None,
            outcome_complete: None,
            abort_return_complete: None,
        }
    }
    // Reserve every recipient queue entry before publishing any state or batch.
    // Physical TCP delivery can still fail; barrier/recipient fences remain required.
    fn publish(&mut self, next: Room, effects: Vec<(u32, Batch)>) -> io::Result<()> {
        let mut reserved = Vec::new();
        for (id, mut batch) in effects {
            let channel = self
                .channels
                .get(&id)
                .ok_or_else(|| err("missing recipient channel"))?;
            let permit = channel
                .sender
                .clone()
                .try_reserve_owned()
                .map_err(|_| err("room recipient queue unavailable"))?;
            batch.origin = Some((self.scope.clone(), id, channel.connection));
            reserved.push((permit, batch));
        }
        if let Some(relay) = &mut self.relay {
            relay.transition(&next)?;
        }
        self.model = next;
        for (permit, batch) in reserved {
            permit.send(batch);
        }
        Ok(())
    }
    fn departure(&self, key: Membership, next: &Room) -> io::Result<Vec<(u32, Batch)>> {
        let departed = self
            .model
            .members
            .get(&key.identity)
            .ok_or_else(|| err("missing departure"))?;
        let remove = packet(107, room_wire::removal(departed)?);
        let mut effects = Vec::new();
        for peer in next.members.values() {
            effects.push((
                peer.key.identity,
                Self::member_batch(
                    peer.key,
                    None,
                    0,
                    vec![
                        ("remove107".into(), remove.clone()),
                        ("survivor110".into(), self.slot(peer, next.host)?),
                    ],
                ),
            ));
        }
        Ok(effects)
    }
    fn forfeit_departure(
        &self,
        key: Membership,
        next: &Room,
        round: u64,
    ) -> io::Result<Vec<(u32, Batch)>> {
        let departed = self
            .model
            .members
            .get(&key.identity)
            .ok_or_else(|| err("missing forfeit departure"))?;
        let removal = packet(107, room_wire::removal(departed)?);
        let alias = if departed.slot == 0 {
            Template::SlotOneWins175
        } else {
            Template::SlotZeroWins175
        };
        let returning = self.reply(
            Template::Return180,
            Some(wire::selector_reply(
                self.body(Template::Return180),
                180,
                self.selector,
            )?),
        );
        let mut effects = Vec::new();
        for peer in next.members.values() {
            // The existing Python recipe waits12s after EOF, then5s rank and3s
            // return. No fabricated19/death report and no reply to the closed peer.
            for (delay, packets, complete) in [
                (
                    12000,
                    vec![("forfeit175".into(), self.reply(alias, None))],
                    false,
                ),
                (17000, vec![("forfeit180".into(), returning.clone())], false),
                (
                    20000,
                    vec![
                        ("forfeit107".into(), removal.clone()),
                        ("forfeit110".into(), self.slot(peer, next.host)?),
                    ],
                    true,
                ),
            ] {
                let mut batch = Self::member_batch(peer.key, Some(round), delay, packets);
                batch.guard = Guard::Outcome(peer.key, round);
                if complete {
                    batch.outcome_complete = Some((peer.key, round));
                }
                effects.push((peer.key.identity, batch));
            }
        }
        Ok(effects)
    }
    fn abort_refresh(&mut self) -> io::Result<()> {
        let Some(job) = self.abort.as_ref() else {
            return Ok(());
        };
        if job.completed
            || job.refresh_queued
            || self.model.members.len() != 3
            || !self
                .model
                .members
                .values()
                .all(|m| job.returned.contains(&m.key) && job.setups.contains(&m.key))
        {
            return Ok(());
        }
        let round = job.round;
        let remove = packet(107, room_wire::removal(&job.departed)?);
        let mut packets = vec![("abort107".into(), remove)];
        for peer in self.model.members.values() {
            packets.push((
                format!("abort110-{}", peer.slot),
                self.slot(peer, self.model.host)?,
            ));
        }
        let mut effects = Vec::new();
        for peer in self.model.members.values() {
            let mut batch = Self::member_batch(peer.key, Some(round), 0, packets.clone());
            batch.guard = Guard::Outcome(peer.key, round);
            batch.outcome_complete = Some((peer.key, round));
            effects.push((peer.key.identity, batch));
        }
        self.publish(self.model.clone(), effects)?;
        self.abort.as_mut().expect("abort job").refresh_queued = true;
        Ok(())
    }
    fn detach(&mut self, id: u32, connection: u64) {
        if self
            .channels
            .get(&id)
            .is_none_or(|c| c.connection != connection)
        {
            return;
        }
        if let Some(relay) = &mut self.relay {
            relay.invalidate();
        }
        if let Ok(m) = self.member(id, connection) {
            let mut next = self.model.clone();
            if self.abort.as_ref().is_some_and(|job| !job.completed) {
                // Invalidate all queued recovery on a second loss. Preserve the
                // first job for provenance; do not claim multi-loss room recovery.
                let _ = next.leave(m.key);
                self.model = next;
                self.channels.clear();
                return;
            }
            if self.rules.departure == DeparturePolicy::UnrankedReturn
                && matches!(next.phase, Phase::Armed { .. })
            {
                let result = (|| -> io::Result<AbortTransition> {
                    let round = next.abort_four(m.key)?;
                    let returning = packet(180, crate::four_results::return_body(self.selector)?);
                    let mut effects = Vec::new();
                    for peer in next.members.values() {
                        let mut batch = Self::member_batch(
                            peer.key,
                            Some(round),
                            0,
                            vec![("abort180".into(), returning.clone())],
                        );
                        batch.guard = Guard::Outcome(peer.key, round);
                        batch.abort_return_complete = Some((peer.key, round));
                        effects.push((peer.key.identity, batch));
                    }
                    let job = FourAbort {
                        round,
                        departed: m.clone(),
                        returned: HashSet::new(),
                        setups: HashSet::new(),
                        refresh_queued: false,
                        completed: false,
                    };
                    Ok((next.clone(), job, effects))
                })();
                match result {
                    Ok((next, job, effects)) => {
                        if self.publish(next.clone(), effects).is_err() {
                            self.model = next;
                            self.channels.clear();
                        }
                        self.abort = Some(job);
                    }
                    Err(_) => {
                        self.channels.clear();
                    }
                }
                self.requested.remove(&m.key);
                self.rostered
                    .retain(|(recipient, subject)| *recipient != m.key && *subject != m.key);
                self.ports.remove(&m.key);
                self.channels.remove(&id);
                return;
            }
            if self.rules.departure == DeparturePolicy::FenceActive
                && matches!(next.phase, Phase::Loading { .. } | Phase::Armed { .. })
            {
                if (if self.managed_round.is_some() {
                    next.fence_active(m.key)
                } else {
                    next.fence_eight(m.key)
                })
                .is_ok()
                {
                    let result = self
                        .departure(m.key, &next)
                        .and_then(|effects| self.publish(next.clone(), effects));
                    if result.is_err() {
                        self.model = next;
                        self.channels.clear();
                    }
                } else {
                    self.channels.clear();
                }
                self.requested.remove(&m.key);
                self.rostered
                    .retain(|(recipient, subject)| *recipient != m.key && *subject != m.key);
                self.ports.remove(&m.key);
                self.channels.remove(&id);
                return;
            }
            let closing_active = self.rules.departure == DeparturePolicy::Forfeit
                && matches!(next.phase, Phase::Armed { .. });
            let transition = if closing_active {
                next.abort_play(m.key).map(Some)
            } else {
                next.leave(m.key).map(|_| None)
            };
            if let Ok(forfeit_round) = transition {
                let effects = if let Some(round) = forfeit_round {
                    self.forfeit_departure(m.key, &next, round)
                } else {
                    self.departure(m.key, &next)
                };
                match effects.and_then(|effects| self.publish(next.clone(), effects)) {
                    Ok(()) => {}
                    Err(_) => {
                        self.model = next;
                        self.channels.clear();
                    }
                }
                self.requested.remove(&m.key);
                self.rostered
                    .retain(|(recipient, subject)| *recipient != m.key && *subject != m.key);
                self.ports.remove(&m.key);
            }
        }
        self.channels.remove(&id);
    }
}
impl Session {
    /// Inspect every account without a room lock or idle renewal. A late DB
    /// completion cannot authorize a different room/round/channel generation.
    pub(crate) async fn pump_relay(&self) -> io::Result<crate::udp_relay::PumpStats> {
        let authorization = {
            let r = self.shared.lock().map_err(|_| err("room unavailable"))?;
            if r.channels
                .get(&self.identity)
                .is_none_or(|c| c.connection != self.connection)
            {
                return Err(err("stale relay channel"));
            }
            r.relay
                .as_ref()
                .and_then(|relay| relay.authorization(&r.model))
        };
        let Some((scope, authorities)) = authorization else {
            return Ok(crate::udp_relay::PumpStats::default());
        };
        for authority in authorities {
            if authority.check_idle().await.is_err() {
                let mut r = self.shared.lock().map_err(|_| err("room unavailable"))?;
                if let Some(relay) = &mut r.relay {
                    relay.invalidate_scope(scope);
                }
                return Ok(crate::udp_relay::PumpStats::default());
            }
        }
        let mut r = self.shared.lock().map_err(|_| err("room unavailable"))?;
        if r.channels
            .get(&self.identity)
            .is_none_or(|c| c.connection != self.connection)
        {
            return Ok(crate::udp_relay::PumpStats::default());
        }
        let model = r.model.clone();
        r.relay
            .as_mut()
            .map_or(Ok(crate::udp_relay::PumpStats::default()), |relay| {
                relay.pump(scope, &model)
            })
    }
    pub fn open(shared: &Shared, identity: u32, connection: u64) -> io::Result<Self> {
        Self::open_with_peer(shared, identity, connection, None)
    }
    pub fn open_with_peer(
        shared: &Shared,
        identity: u32,
        connection: u64,
        peer: Option<std::net::Ipv4Addr>,
    ) -> io::Result<Self> {
        Self::open_with_authority(shared, identity, connection, peer, None)
    }
    pub(crate) fn open_with_authority(
        shared: &Shared,
        identity: u32,
        connection: u64,
        peer: Option<std::net::Ipv4Addr>,
        relay_authority: Option<crate::account_bridge::RelayAuthority>,
    ) -> io::Result<Self> {
        let mut runtime = shared.lock().map_err(|_| err("room unavailable"))?;
        if runtime.channels.contains_key(&identity) {
            return Err(err("duplicate room channel"));
        }
        let (sender, inbox) = mpsc::channel(rules::recipient_queue(runtime.model.capacity()));
        let endpoint = peer
            .map(u32::from)
            .or_else(|| runtime.ips.get(&identity).copied())
            .unwrap_or(0);
        runtime.channels.insert(
            identity,
            Channel {
                connection,
                sender,
                endpoint,
                relay_authority,
            },
        );
        if runtime.public_key.is_none() && runtime.model.host.is_some() {
            let batch = runtime.lobby_batch(
                identity,
                connection,
                0,
                vec![("listing91".into(), runtime.reply(Template::Listing91, None))],
            );
            let unchanged = runtime.model.clone();
            if runtime.publish(unchanged, vec![(identity, batch)]).is_err() {
                runtime.channels.remove(&identity);
                return Err(err("initial listing queue unavailable"));
            }
        }
        Ok(Self {
            shared: shared.clone(),
            inbox,
            identity,
            connection,
        })
    }
    /// Ancillary lobby observations must not be accepted during room/Battle.
    pub fn in_lobby(&self) -> bool {
        self.shared.lock().is_ok_and(|runtime| {
            runtime
                .channels
                .get(&self.identity)
                .is_some_and(|channel| channel.connection == self.connection)
                && !runtime.model.members.contains_key(&self.identity)
        })
    }
    pub fn tick(&self) -> io::Result<()> {
        let mut r = self.shared.lock().map_err(|_| err("room unavailable"))?;
        if r.abort.is_some() {
            return r.abort_refresh();
        }
        if r.rules.result == ResultPolicy::Disabled {
            return Ok(());
        }
        let Phase::Armed { round } = r.model.phase else {
            return Ok(());
        };
        if !r.model.valid_round_roster() {
            return Ok(());
        }
        let now = Instant::now();
        let rank = if r.rules.team_results() {
            let Some(team) = crate::four_results::winner(&r.model, now) else {
                return Ok(());
            };
            packet(
                175,
                crate::four_results::rank_body(r.model.round_teams(), team)?,
            )
        } else {
            let members: Vec<_> = r.model.members.values().collect();
            let dead: Vec<_> = members
                .iter()
                .filter_map(|m| m.life.death.map(|at| (m.slot, at)))
                .collect();
            let alias = if dead.len() == 2 {
                if dead
                    .iter()
                    .any(|(_, at)| now.duration_since(*at) < Duration::from_secs(8))
                {
                    return Ok(());
                }
                Template::AllDead175
            } else if let Some(&(slot, at)) = dead.first() {
                if now.duration_since(at) < Duration::from_secs(8)
                    || r.model.bombs.pending()
                    || !members.iter().any(|m| {
                        m.slot != slot
                            && m.life.survivor.is_some_and(|s| {
                                s > at && m.life.current_survivor(s, r.model.last_blast)
                            })
                    })
                {
                    return Ok(());
                }
                if slot == 0 {
                    Template::SlotOneWins175
                } else {
                    Template::SlotZeroWins175
                }
            } else {
                return Ok(());
            };
            r.reply(alias, None)
        };
        let mut next = r.model.clone();
        next.end(round)?;
        let mut effects = Vec::new();
        let returning = if r.rules.team_results() {
            packet(180, crate::four_results::return_body(r.selector)?)
        } else {
            r.reply(
                Template::Return180,
                Some(wire::selector_reply(
                    r.body(Template::Return180),
                    180,
                    r.selector,
                )?),
            )
        };
        let slots = next
            .members
            .values()
            .map(|m| r.slot(m, next.host))
            .collect::<io::Result<Vec<_>>>()?;
        for peer in next.members.values() {
            for (delay, packets, complete) in [
                (0, vec![("rank175".into(), rank.clone())], false),
                (5000, vec![("return180".into(), returning.clone())], false),
                (
                    8000,
                    slots
                        .iter()
                        .enumerate()
                        .map(|(i, p)| (format!("refresh110-{i}"), p.clone()))
                        .collect(),
                    true,
                ),
            ] {
                let mut batch = Runtime::member_batch(peer.key, Some(round), delay, packets);
                batch.guard = Guard::Outcome(peer.key, round);
                if complete {
                    batch.outcome_complete = Some((peer.key, round));
                }
                effects.push((peer.key.identity, batch));
            }
        }
        r.publish(next, effects)
    }
    pub fn handles(opcode: u32) -> bool {
        matches!(
            opcode,
            81 | 82
                | 585
                | 86
                | 95
                | 104
                | 99
                | 528
                | 127
                | 96
                | 266
                | 190
                | 124
                | 123
                | 118
                | 142
                | 167
                | 202
        )
    }
    pub fn valid(&self, batch: &Batch) -> bool {
        let Ok(r) = self.shared.lock() else {
            return false;
        };
        if !batch
            .origin
            .as_ref()
            .is_some_and(|(scope, identity, connection)| {
                Arc::ptr_eq(scope, &r.scope)
                    && *identity == self.identity
                    && *connection == self.connection
            })
        {
            return false;
        }
        if r.channels
            .get(&self.identity)
            .is_none_or(|c| c.connection != self.connection)
        {
            return false;
        }
        if batch.subject.is_some_and(|key| !r.model.current(key)) {
            return false;
        }
        match batch.guard {
            Guard::Channel => true,
            Guard::Bomb { member, ref ticket } => {
                r.model.deliverable(member, Some(ticket.round))
                    && r.model.bombs.deliverable(ticket, member)
            }
            Guard::Member(key, round) => r.model.deliverable(key, round),
            Guard::Outcome(key, round) => r.model.outcome_deliverable(key, round),
            Guard::Life {
                recipient,
                source,
                round,
                ref clear21,
            } => {
                r.model.deliverable(recipient, Some(round))
                    && r.model.current(source)
                    && matches!(r.model.phase,Phase::Armed { round:current } if current==round)
                    && clear21.as_ref().is_none_or(|(digest, at)| {
                        r.model.members[&source.identity].life.forward21_valid(
                            digest,
                            *at,
                            r.model.last_blast,
                        )
                    })
            }
            Guard::Lobby {
                identity,
                connection,
                visit,
            } => {
                identity == self.identity
                    && connection == self.connection
                    && !r.model.members.contains_key(&identity)
                    && *r.last_visits.get(&identity).unwrap_or(&0) == visit
            }
        }
    }
    pub fn project(&self, reply: &Reply) -> io::Result<Option<Reply>> {
        let r = self.shared.lock().map_err(|_| err("room unavailable"))?;
        match reply.opcode {
            106 => {
                let id =
                    read_uint(&reply.payload[1..5], Width::U32).map_err(io::Error::other)? as u32;
                match r.model.members.get(&id) {
                    Some(m) => Ok(Some(r.player(m, r.model.host)?)),
                    None => Ok(None),
                }
            }
            110 => {
                let slot =
                    read_uint(&reply.payload[..1], Width::U8).map_err(io::Error::other)? as u8;
                match r.model.members.values().find(|m| m.slot == slot) {
                    Some(m) => Ok(Some(r.slot(m, r.model.host)?)),
                    None => Ok(None),
                }
            }
            _ => Ok(Some(reply.clone())),
        }
    }
    pub fn finish(&self, batch: &Batch) -> io::Result<()> {
        if let Some((source, round, digest, recipient)) = &batch.life_complete {
            let mut runtime = self.shared.lock().map_err(|_| err("room unavailable"))?;
            if runtime.model.current(*source)
                && runtime.model.current(*recipient)
                && runtime.model.phase == (Phase::Armed { round: *round })
            {
                runtime
                    .model
                    .members
                    .get_mut(&source.identity)
                    .expect("current source")
                    .life
                    .report_delivered(digest, *recipient);
            }
        }
        if let Some((recipient, round)) = batch.abort_return_complete {
            let mut runtime = self.shared.lock().map_err(|_| err("room unavailable"))?;
            if !runtime.model.outcome_deliverable(recipient, round) {
                return Err(err("stale abort return completion"));
            }
            let job = runtime
                .abort
                .as_mut()
                .filter(|j| j.round == round && !j.completed)
                .ok_or_else(|| err("missing abort return job"))?;
            if !job.returned.insert(recipient) {
                return Err(err("duplicate abort return completion"));
            }
        }
        if let Some((recipient, round)) = batch.outcome_complete {
            let mut runtime = self.shared.lock().map_err(|_| err("room unavailable"))?;
            runtime.model.refreshed(recipient, round)?;
            if runtime.model.phase == Phase::Idle
                && let Some(job) = runtime.abort.as_mut()
            {
                job.completed = true;
            }
        }
        if let Some((ticket, recipient)) = &batch.bomb_complete {
            let mut runtime = self.shared.lock().map_err(|_| err("room unavailable"))?;
            if !runtime.model.bombs.delivered(ticket, *recipient) {
                return Err(err("stale bomb delivery completion"));
            }
            if !runtime.model.bombs.active(ticket) {
                runtime.model.last_blast = Some(Instant::now());
                for member in runtime.model.members.values_mut() {
                    member.life.blast_committed();
                }
                if let Some(terrain) = &mut runtime.model.terrain {
                    terrain.blast_committed(ticket.round, ticket.id);
                }
            }
        }

        if let Some(key) = batch.setup {
            self.shared
                .lock()
                .map_err(|_| err("room unavailable"))?
                .model
                .setup(key)?;
        }
        Ok(())
    }
    pub fn packet(&self, opcode: u32, body: &[u8]) -> io::Result<Observation> {
        let mut r = self.shared.lock().map_err(|_| err("room unavailable"))?;
        if r.channels
            .get(&self.identity)
            .is_none_or(|c| c.connection != self.connection)
        {
            return Err(err("stale room channel"));
        }
        if r.public_key.is_some()
            && r.managed_round.is_none()
            && !matches!(opcode, 81 | 82 | 585 | 86 | 528 | 96 | 266)
        {
            return Err(err("managed room selection does not yet serve gameplay"));
        }
        let mut next = r.model.clone();
        let mut effects = Vec::new();
        let mut new_key = None;
        let mut requested = None;
        let mut remove_requested = None;
        let mut loaded_endpoint = None;
        let mut roster_pairs = Vec::new();
        let mut room_choice = None;
        let mut dropped_after_abort = false;
        let mut pickup = None;
        let mut bomb = None;
        let mut terrain_notice = None;
        let mut life = None;
        match opcode {
            528 => {
                if !body.is_empty() {
                    return Err(err("cache528 must be empty"));
                }
                let mut batch = r.lobby_batch(
                    self.identity,
                    self.connection,
                    0,
                    vec![("table238".into(), r.reply(Template::Table238, None))],
                );
                batch.guard = Guard::Channel;
                effects.push((self.identity, batch));
            }
            81 | 82 => {
                let (title, selector) = if opcode == 81 {
                    (wire::create81_title(body)?, wire::create81_selector(body)?)
                } else {
                    if let Some(key) = r.public_key {
                        let selection = wire::join82_selection(body)?;
                        if selection.session != key.session || selection.room_id != key.id.0 {
                            return Err(err("room selection context mismatch"));
                        }
                    } else {
                        wire::join82_context(body)?;
                    }
                    (r.title.clone(), r.selector)
                };
                if r.rules.ninth_admission_probe && self.identity == 9 {
                    // CP227 generic2 is native-confirmed, capacity is our design reason.
                    // This probe never acquires a room slot/visit/round. Vacancy and
                    // replacement admission remain outside this fixed diagnostic profile.
                    if opcode != 82 || next.members.len() != 8 || next.phase != Phase::Idle {
                        return Err(err("ninth probe requires full idle eight-room"));
                    }
                    let body = kr_wire::write_uint(2, Width::U8).map_err(io::Error::other)?;
                    let batch = r.lobby_batch(
                        self.identity,
                        self.connection,
                        0,
                        vec![("full87".into(), r.reply(Template::Join87, Some(body)))],
                    );
                    r.publish(next, vec![(self.identity, batch)])?;
                    return Ok(Observation {
                        admission_rejection: Some("diagnostic_eight_room_full"),
                        ..Observation::default()
                    });
                }
                let key = next.enter(self.identity, self.connection, opcode == 81)?;
                let m = next.members[&self.identity].clone();
                let reply = if opcode == 81 {
                    r.reply(
                        Template::Create86,
                        Some(wire::selector_reply(
                            r.body(Template::Create86),
                            86,
                            selector,
                        )?),
                    )
                } else {
                    r.reply(
                        Template::Join87,
                        Some(wire::selector_reply(
                            &room_wire::join(r.body(Template::Join87), &m)?,
                            87,
                            selector,
                        )?),
                    )
                };
                let reply = if let Some(public) = r.public_key {
                    let payload = if opcode == 81 {
                        wire::create86_session(&reply.payload, public.session)?
                    } else {
                        wire::join87_session(&reply.payload, m.slot, public.session)?
                    };
                    Reply { payload, ..reply }
                } else {
                    reply
                };
                effects.push((
                    self.identity,
                    Runtime::member_batch(
                        key,
                        None,
                        0,
                        vec![(
                            if opcode == 81 { "create86" } else { "join87" }.into(),
                            reply,
                        )],
                    ),
                ));
                if opcode == 81 {
                    for (&id, c) in &r.channels {
                        if id != self.identity && !next.members.contains_key(&id) {
                            effects.push((
                                id,
                                r.lobby_batch(
                                    id,
                                    c.connection,
                                    0,
                                    vec![(
                                        "listing91".into(),
                                        r.reply(
                                            Template::Listing91,
                                            Some(wire::listing91_replace_title(
                                                r.body(Template::Listing91),
                                                &title,
                                            )?),
                                        ),
                                    )],
                                ),
                            ));
                        }
                    }
                }
                room_choice = Some((title, selector));
                new_key = Some(key);
            }
            585 => {
                if !wire::zero585_setup(body)? {
                    return Err(err("unsupported setup585"));
                }
                let m = r.member(self.identity, self.connection)?;
                if let Some(job) = r.abort.as_ref().filter(|j| !j.completed) {
                    if !job.returned.contains(&m.key) {
                        return Err(err("abort setup585 before completed return180"));
                    }
                    next.observe_closed(m.key)?; // bounded duplicate/tail budget
                    r.publish(next, vec![])?;
                    r.abort.as_mut().expect("abort job").setups.insert(m.key);
                    return Ok(Observation {
                        dropped_after_abort: true,
                        ..Observation::default()
                    });
                }
                if r.requested.contains(&m.key) {
                    // Native180 sends585 again. The retained Python profile ignores
                    // this tail;110 refresh owns the return. Raw585 has no epoch.
                    if matches!(next.phase, Phase::Ending { .. }) {
                        next.observe_closed(m.key)?;
                    } else if next.phase == Phase::Idle {
                        next.observe_after_abort(m.key)?;
                    } else {
                        return Err(err("duplicate setup585"));
                    }
                    r.publish(next, vec![])?;
                    return Ok(Observation {
                        dropped_after_abort: true,
                        ..Observation::default()
                    });
                }
                effects.push((
                    self.identity,
                    Runtime::member_batch(
                        m.key,
                        None,
                        0,
                        vec![("own106".into(), r.player(&m, next.host)?)],
                    ),
                ));
                effects.push((
                    self.identity,
                    Runtime::member_batch(
                        m.key,
                        None,
                        2000,
                        vec![("own110".into(), r.slot(&m, next.host)?)],
                    ),
                ));
                for peer in next.members.values() {
                    if peer.key != m.key {
                        if !r.rostered.contains(&(m.key, peer.key)) {
                            let mut batch = Runtime::member_batch(
                                m.key,
                                None,
                                3000,
                                vec![("peer106".into(), r.player(peer, next.host)?)],
                            );
                            batch.subject = Some(peer.key);
                            effects.push((self.identity, batch));
                            roster_pairs.push((m.key, peer.key));
                        }
                        if r.requested.contains(&peer.key)
                            && !r.rostered.contains(&(peer.key, m.key))
                        {
                            let mut batch = Runtime::member_batch(
                                peer.key,
                                None,
                                3000,
                                vec![("peer106".into(), r.player(&m, next.host)?)],
                            );
                            batch.subject = Some(m.key);
                            roster_pairs.push((peer.key, m.key));
                            effects.push((peer.key.identity, batch));
                        }
                    }
                }
                let mut final_batch = Runtime::member_batch(
                    m.key,
                    None,
                    4000,
                    vec![
                        ("table238".into(), r.reply(Template::Table238, None)),
                        ("map117".into(), r.reply(Template::Map117, None)),
                    ],
                );
                final_batch.setup = Some(m.key);
                effects.push((self.identity, final_batch));
                requested = Some(m.key);
            }
            86 => {
                if !body.is_empty() {
                    return Err(err("leave86 must be empty"));
                }
                let m = r.member(self.identity, self.connection)?;
                next.leave(m.key)?;
                effects.extend(r.departure(m.key, &next)?);
                effects.push((
                    self.identity,
                    r.lobby_batch(
                        self.identity,
                        self.connection,
                        1000,
                        vec![
                            ("return241".into(), r.ready.clone()),
                            ("return91".into(), r.reply(Template::Listing91, None)),
                        ],
                    ),
                ));
                remove_requested = Some(m.key);
            }
            95 => {
                let m = r.member(self.identity, self.connection)?;
                if body.len() < 9 {
                    return Err(err("Ready95 envelope"));
                }
                let n = read_uint(&body[..4], Width::U32).map_err(io::Error::other)? as usize;
                if !(6..=8192).contains(&n)
                    || body.len() != n + 9
                    || read_uint(&body[body.len() - 5..body.len() - 4], Width::U8)
                        .map_err(io::Error::other)?
                        != 1
                    || read_uint(&body[body.len() - 4..], Width::U32).map_err(io::Error::other)?
                        != 0
                {
                    return Err(err("Ready95 shape"));
                }
                next.ready(m.key)?;
                let member = next.members[&self.identity].clone();
                for peer in next.members.values() {
                    effects.push((
                        peer.key.identity,
                        Runtime::member_batch(
                            peer.key,
                            None,
                            0,
                            vec![("ready110".into(), r.slot(&member, next.host)?)],
                        ),
                    ));
                }
            }
            104 => {
                let m = r.member(self.identity, self.connection)?;
                if !(20..=8192).contains(&body.len()) {
                    return Err(err("unsupported Start104 envelope"));
                }
                if r.rules.teams == TeamPolicy::BalancedManner
                    && r.selector == 1
                    && !next.balanced_teams()
                {
                    return Ok(Observation {
                        start_rejection: Some("manner_fixed_teams_unbalanced"),
                        ..Observation::default()
                    });
                }
                let round = next.begin(m.key)?;
                next.pickups = pickups::Round::begin(round, &r.floor_catalog)?;
                if let Some(profile) = &r.managed_round {
                    for member in next.members.values() {
                        next.bomb_capacity[usize::from(member.slot)] =
                            profile.initial_bomb_capacity[usize::from(member.slot)];
                    }
                }
                if r.rules.endpoint == rules::EndpointPolicy::Relay {
                    let rows = next
                        .members
                        .values()
                        .map(|member| {
                            let channel = r
                                .channels
                                .get(&member.key.identity)
                                .filter(|c| c.connection == member.key.connection)
                                .ok_or_else(|| err("missing relay channel owner"))?;
                            let authority = channel
                                .relay_authority
                                .clone()
                                .ok_or_else(|| err("relay requires account-owned channels"))?;
                            Ok((
                                member.slot,
                                member.key,
                                std::net::Ipv4Addr::from(channel.endpoint),
                                authority,
                            ))
                        })
                        .collect::<io::Result<Vec<_>>>()?;
                    let relay = r.relay.as_mut().ok_or_else(|| err("missing relay pool"))?;
                    let reservation = relay.prepare(&next, rows)?;
                    relay.install(reservation);
                }
                for peer in next.members.values() {
                    effects.push((
                        peer.key.identity,
                        Runtime::member_batch(
                            peer.key,
                            Some(round),
                            0,
                            vec![
                                ("floor582".into(), r.reply(Template::Floor582, None)),
                                ("start121".into(), r.reply(Template::Start121, None)),
                            ],
                        ),
                    ));
                }
            }
            99 => {
                let m = r.member(self.identity, self.connection)?;
                if body.len() != 23 {
                    return Err(err("unsupported loading99 size"));
                }
                let port = read_uint(&body[8..10], Width::U16).map_err(io::Error::other)? as u16;
                if port == 0 {
                    return Err(err("loading UDP port zero"));
                }
                if next.phase == Phase::Idle {
                    next.observe_after_abort(m.key)?;
                    dropped_after_abort = true;
                } else {
                    let Phase::Loading { round, .. } = next.phase else {
                        return Err(err("no loading barrier"));
                    };
                    let complete = next.ack(m.key, round)?;
                    loaded_endpoint = Some((m.key, port));
                    if complete {
                        let mut endpoint_rows = Vec::with_capacity(next.round_size());
                        for peer in next.members.values() {
                            let p = if peer.key == m.key {
                                port
                            } else {
                                *r.ports
                                    .get(&peer.key)
                                    .ok_or_else(|| err("missing current loading endpoint"))?
                            };
                            let channel = r
                                .channels
                                .get(&peer.key.identity)
                                .filter(|channel| channel.connection == peer.key.connection)
                                .ok_or_else(|| err("missing current endpoint channel"))?;
                            endpoint_rows.push((peer.key, peer.slot, channel.endpoint, p));
                        }
                        let plan = crate::peer_routes::LegacyDirectPlan::new(
                            round,
                            next.round_size(),
                            endpoint_rows,
                        )?;
                        let key = crate::room::wire_key(round)?;
                        for peer in next.members.values() {
                            effects.push((
                                peer.key.identity,
                                Runtime::member_batch(
                                    peer.key,
                                    Some(round),
                                    0,
                                    vec![("go197".into(), r.reply(Template::Go197, None))],
                                ),
                            ));
                            effects.push((
                                peer.key.identity,
                                Runtime::member_batch(
                                    peer.key,
                                    Some(round),
                                    1000,
                                    vec![("play196".into(), r.reply(Template::Play196, None))],
                                ),
                            ));
                            effects.push((
                                peer.key.identity,
                                Runtime::member_batch(
                                    peer.key,
                                    Some(round),
                                    2000,
                                    vec![(
                                        "peer153".into(),
                                        r.reply(
                                            Template::Peer153,
                                            Some(wire::peer153(
                                                r.body(Template::Peer153),
                                                key,
                                                peer.slot,
                                                &if r.rules.endpoint == rules::EndpointPolicy::Relay
                                                {
                                                    r.relay
                                                        .as_ref()
                                                        .ok_or_else(|| {
                                                            err("missing relay reservation")
                                                        })?
                                                        .endpoints(peer.key)?
                                                } else {
                                                    plan.for_recipient(round, peer.key)?.to_vec()
                                                },
                                            )?),
                                        ),
                                    )],
                                ),
                            ));
                        }
                    }
                }
            }
            190 => {
                let m = r.member(self.identity, self.connection)?;
                if !body.is_empty() {
                    return Err(err("known190 observation shape"));
                }
                if next.phase == Phase::Idle {
                    next.observe_after_abort(m.key)?;
                    dropped_after_abort = true;
                } else if matches!(next.phase, Phase::Ending { .. }) {
                    next.observe_closed(m.key)?;
                    dropped_after_abort = true;
                } else if !matches!(next.phase, Phase::Armed { .. }) {
                    return Err(err("known190 observation phase"));
                }
            }
            96 | 266 => {
                r.member(self.identity, self.connection)?;
                if (opcode == 96 && body.len() != 6) || (opcode == 266 && body.len() != 12) {
                    return Err(err("known room ancillary shape mismatch"));
                }
                // CP019/020 known map-selection/setup traffic; Python leaves
                // these shapes unanswered. No new authority/side effect.
            }
            202 => {
                let member = r.member(self.identity, self.connection)?;
                if body.len() != 22
                    || read_uint(&body[..1], Width::U8).map_err(io::Error::other)? != 0
                    || read_uint(&body[1..2], Width::U8).map_err(io::Error::other)? != 1
                {
                    return Err(err("unsupported closed-result202 envelope"));
                }
                if matches!(next.phase, Phase::Ending { .. }) {
                    next.observe_closed(member.key)?;
                } else if next.phase == Phase::Idle {
                    next.observe_after_abort(member.key)?;
                } else {
                    return Err(err("active battle exit202 not ported"));
                }
                dropped_after_abort = true;
            }
            167 => {
                let member = r.member(self.identity, self.connection)?;
                let measured = if let Some(profile) = &r.managed_round {
                    profile.limits.opaque167.contains(&body.len())
                } else if next.capacity() == 8 {
                    r.rules.owner_bombs() && [4, 22, 42].contains(&body.len())
                } else {
                    [4, 16, 32].contains(&body.len())
                        || (next.capacity() == 4 && r.rules.owner_bombs() && body.len() == 22)
                };
                if !measured {
                    return Err(err("opaque167 measured sizes"));
                }
                if matches!(next.phase, Phase::Ending { .. }) {
                    next.observe_closed(member.key)?;
                    dropped_after_abort = true;
                } else if next.phase == Phase::Idle {
                    next.observe_after_abort(member.key)?;
                    dropped_after_abort = true;
                } else {
                    next.observe_opaque167(member.key)?;
                }
            }
            123 => {
                let m = r.member(self.identity, self.connection)?;
                if body.len() != 10 {
                    return Err(err("opaque123 measured size"));
                }
                if matches!(next.phase, Phase::Ending { .. }) {
                    next.observe_closed(m.key)?;
                    dropped_after_abort = true;
                } else if next.phase == Phase::Idle {
                    next.observe_after_abort(m.key)?;
                    dropped_after_abort = true;
                } else {
                    next.observe_opaque123(m.key)?;
                }
            }
            118 => {
                if r.rules.bomb == rules::BombPolicy::Disabled {
                    return Err(err("four-player gameplay not ported"));
                }
                let member = r.member(self.identity, self.connection)?;
                let request = bombs::Request::parse(body)?;
                let result = if r.managed_round.as_ref().is_some_and(|profile| {
                    profile.limits.power.get(usize::from(member.slot)) != Some(&request.power())
                }) {
                    Err("unsupported_profile_power")
                } else if r.managed_round.is_none()
                    && ((next.capacity() == 4 && request.power() != 1)
                        || (next.capacity() == 8
                            && request.power()
                                != [1, 1, 1, 1, 1, 1, 2, 2][usize::from(member.slot)]))
                {
                    Err("unsupported_four_power")
                } else if matches!(next.phase, Phase::Armed { .. })
                    && next.members[&self.identity].life.death.is_some()
                {
                    Err("dead_owner")
                } else if let Phase::Armed { round } = next.phase {
                    let roster: Vec<_> = next.members.values().map(|m| m.key).collect();
                    next.bombs.place(
                        round,
                        member.key,
                        member.slot,
                        &request,
                        &roster,
                        next.bomb_capacity[usize::from(member.slot)],
                    )?
                } else {
                    Err("inactive_round")
                };
                match result {
                    Err(reason) => {
                        bomb = Some(BombObservation {
                            accepted: false,
                            id: None,
                            reason,
                        })
                    }
                    Ok(ticket) => {
                        if let Some(terrain) = &mut next.terrain
                            && terrain.place(request.terrain_bomb(ticket.id)).is_err()
                        {
                            // Discard every candidate allocation/serial/reservation.
                            return Ok(Observation {
                                bomb: Some(BombObservation {
                                    accepted: false,
                                    id: None,
                                    reason: "terrain_placement",
                                }),
                                ..Observation::default()
                            });
                        }
                        let spawn = packet(159, request.spawn159(ticket.id)?);
                        let blast = packet(158, bombs::explode158(ticket.id)?);
                        for peer in next.members.values() {
                            let mut immediate = Runtime::member_batch(
                                peer.key,
                                Some(ticket.round),
                                0,
                                vec![("bomb159".into(), spawn.clone())],
                            );
                            immediate.subject = Some(member.key);
                            effects.push((peer.key.identity, immediate));
                            let mut due = Runtime::member_batch(
                                peer.key,
                                Some(ticket.round),
                                6000,
                                vec![("bomb158".into(), blast.clone())],
                            );
                            due.guard = Guard::Bomb {
                                member: peer.key,
                                ticket: ticket.clone(),
                            };
                            due.subject = Some(member.key);
                            due.bomb_complete = Some((ticket.clone(), peer.key));
                            effects.push((peer.key.identity, due));
                        }
                        bomb = Some(BombObservation {
                            accepted: true,
                            id: Some(ticket.id),
                            reason: "accepted",
                        });
                    }
                }
            }
            142 => {
                if r.rules.bomb == rules::BombPolicy::Disabled {
                    return Err(err("four-player gameplay not ported"));
                }
                let member = r.member(self.identity, self.connection)?;
                if body.len() != 7 {
                    return Err(err("known142 size"));
                }
                let slot = read_uint(&body[2..3], Width::U8).map_err(io::Error::other)?;
                let aux = read_uint(&body[3..], Width::U32).map_err(io::Error::other)?;
                terrain_notice = Some(if matches!(next.capacity(), 4 | 8) {
                    // CP201/209 actual142 follows explosions, including aux37 and -1.
                    // Observe the measured shape only. No reveal/eligibility/item effect.
                    let row = read_uint(&body[..1], Width::U8).map_err(io::Error::other)?;
                    let col = read_uint(&body[1..2], Width::U8).map_err(io::Error::other)?;
                    if slot != u64::from(member.slot)
                        || row >= 13
                        || col >= 15
                        || !(if let Some(profile) = &r.managed_round {
                            profile.limits.terrain_aux.contains(&aux)
                        } else if next.capacity() == 8 {
                            [33, 37, 39, 47, u64::from(u32::MAX)].contains(&aux)
                        } else {
                            [37, u64::from(u32::MAX)].contains(&aux)
                        })
                    {
                        return Err(err("unsupported four bomb-only142 report"));
                    }
                    next.observe_opaque142(member.key)?;
                    "four_bomb_only_observed_no_reveal"
                } else if let Phase::Armed { round } = next.phase {
                    let row = read_uint(&body[..1], Width::U8).map_err(io::Error::other)? as u8;
                    let col = read_uint(&body[1..2], Width::U8).map_err(io::Error::other)? as u8;
                    if slot != u64::from(member.slot) || aux != u64::from(u32::MAX) {
                        "unsupported_report"
                    } else if !next.pickups.hidden_cell(row, col) {
                        "unknown_hidden_cell"
                    } else if next
                        .terrain
                        .as_ref()
                        .and_then(|t| t.clearance(round, row, col))
                        .is_none()
                    {
                        "no_current_committed_clearance"
                    } else if next.pickups.reveal(round, row, col) {
                        "revealed_current_hidden_cell"
                    } else {
                        "already_revealed"
                    }
                } else {
                    "inactive_round"
                });
            }
            124 => {
                if r.managed_round.is_some() || next.capacity() != 2 {
                    return Err(err("four-player gameplay not ported"));
                }
                let member = r.member(self.identity, self.connection)?;
                let request = pickups::Request::parse(body)?;
                let decision = if next.members[&self.identity].life.blocked_pickup() {
                    request.decision("report_ineligible")
                } else if let Phase::Armed { round } = next.phase {
                    next.pickups.claim(round, member.slot, &request)
                } else {
                    request.decision("inactive_round")
                };
                if decision.accepted {
                    if request.kind() == 0 {
                        let slot = usize::from(member.slot);
                        let maximum = [10, 7][slot];
                        next.bomb_capacity[slot] = (next.bomb_capacity[slot] + 1).min(maximum);
                    }
                    let Phase::Armed { round } = next.phase else {
                        unreachable!("accepted only in armed round")
                    };
                    let reply = packet(165, request.confirmation()?);
                    for peer in next.members.values() {
                        let mut batch = Runtime::member_batch(
                            peer.key,
                            Some(round),
                            0,
                            vec![("floor165".into(), reply.clone())],
                        );
                        batch.subject = Some(member.key);
                        effects.push((peer.key.identity, batch));
                    }
                }
                pickup = Some(decision);
            }
            127 => {
                let m = r.member(self.identity, self.connection)?;
                if !(1..=4096).contains(&body.len()) {
                    return Err(err("report observation size"));
                }
                if next.phase == Phase::Idle {
                    next.observe_after_abort(m.key)?;
                    dropped_after_abort = true;
                } else if matches!(next.phase, Phase::Ending { .. }) {
                    // Closed-round transport observation only; no cursor, world or result mutation.
                    next.observe_closed(m.key)?;
                    dropped_after_abort = true;
                } else if let Phase::Armed { round } = next.phase {
                    if r.rules.life.reports_enabled() {
                        let report = crate::life::Report::parse(body)?;
                        let now = Instant::now();
                        let last_blast = next.last_blast;
                        let decision =
                            if r.rules.life_enabled() && !matches!(report.subtype, 17 | 19 | 21) {
                                report.decision("unsupported_four_report_subtype", false)
                            } else if r.rules.life_enabled()
                                && next.members[&self.identity].life.death.is_some()
                            {
                                report.decision("report_after_accepted_death", false)
                            } else {
                                next.members
                                    .get_mut(&self.identity)
                                    .expect("current member")
                                    .life
                                    .apply(m.slot, &report, now, last_blast)
                            };
                        if decision.forward {
                            let recipients: Vec<_> = next
                                .members
                                .values()
                                .filter(|p| p.key != m.key)
                                .map(|p| p.key)
                                .collect();
                            if recipients.len() + 1 != next.round_size() {
                                return Err(err("incomplete life recipient roster"));
                            }
                            if report.subtype == 21 {
                                next.members
                                    .get_mut(&self.identity)
                                    .expect("current source")
                                    .life
                                    .forward21_targets(&report.digest, &recipients)?;
                            } else {
                                next.members
                                    .get_mut(&self.identity)
                                    .expect("current source")
                                    .life
                                    .forward_report_targets(&report.digest, &recipients)?;
                            }
                            for recipient in recipients {
                                let mut batch = Runtime::member_batch(
                                    recipient,
                                    Some(round),
                                    0,
                                    vec![(
                                        format!("life170-{}", report.subtype),
                                        packet(
                                            170,
                                            report.forward.clone().ok_or_else(|| {
                                                err("missing validated life170 body")
                                            })?,
                                        ),
                                    )],
                                );
                                batch.subject = Some(m.key);
                                batch.guard = Guard::Life {
                                    recipient,
                                    source: m.key,
                                    round,
                                    clear21: (report.subtype == 21)
                                        .then(|| (report.digest.clone(), now)),
                                };
                                batch.life_complete =
                                    Some((m.key, round, report.digest.clone(), recipient));
                                effects.push((recipient.identity, batch));
                            }
                        }
                        life = Some(decision);
                    }
                } else {
                    return Err(err("report observation phase"));
                }
            }
            _ => return Err(err("unsupported room request")),
        }
        r.publish(next, effects)?;
        if let Some(key) = new_key {
            r.last_visits.insert(self.identity, key.visit);
        }
        if let Some(key) = requested {
            r.requested.insert(key);
        }
        if let Some(key) = remove_requested {
            r.requested.remove(&key);
            r.ports.remove(&key);
        }
        if let Some((title, selector)) = room_choice {
            let listing = wire::listing91_replace_title(r.body(Template::Listing91), &title)?;
            r.bodies.insert(Template::Listing91, listing);
            r.title = title;
            r.selector = selector;
        }
        for pair in roster_pairs {
            r.rostered.insert(pair);
        }
        if let Some(key) = remove_requested {
            r.rostered
                .retain(|(recipient, subject)| *recipient != key && *subject != key);
        }
        if let Some((key, port)) = loaded_endpoint {
            r.ports.insert(key, port);
        }
        Ok(Observation {
            start_rejection: None,
            admission_rejection: None,
            dropped_after_abort,
            pickup,
            bomb,
            terrain_notice,
            life,
        })
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        if let Ok(mut r) = self.shared.lock() {
            r.detach(self.identity, self.connection);
        }
    }
}

#[cfg(test)]
fn synthetic_players(packets: &HashMap<String, Reply>) -> HashMap<PlayerId, Reply> {
    packets
        .values()
        .filter(|packet| packet.opcode == 106)
        .map(|packet| {
            (
                read_uint(&packet.payload[1..5], Width::U32).unwrap() as PlayerId,
                packet.clone(),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn outcome_queue_failure_is_atomic_and_old_refresh_cannot_open_new_round() {
        use crate::life::Report;
        use kr_wire::write_uint;
        let packet = |op, payload| Reply {
            header: [0; 37],
            opcode: op,
            payload,
        };
        let mut packets = HashMap::new();
        for (alias, id) in [("player_A", 1u64), ("player_B", 2)] {
            let mut payload = vec![0; 204];
            payload[1..5].copy_from_slice(&write_uint(id, Width::U32).unwrap());
            packets.insert(alias.into(), packet(106, payload));
        }
        for alias in ["rank", "rank_Awin", "rank_all_dead"] {
            packets.insert(alias.into(), packet(175, vec![0; 302]));
        }
        packets.insert(
            "return".into(),
            packet(180, write_uint(0, Width::U8).unwrap().repeat(3)),
        );
        let mut model = Room::default();
        let a = model.enter(1, 11, true).unwrap();
        let b = model.enter(2, 22, false).unwrap();
        model.setup(a).unwrap();
        model.setup(b).unwrap();
        model.ready(b).unwrap();
        let round = model.begin(a).unwrap();
        model.ack(a, round).unwrap();
        model.ack(b, round).unwrap();
        let dead_at = Instant::now() - Duration::from_secs(10);
        for (subtype, tick, now) in [(17, 1, dead_at - Duration::from_secs(1)), (19, 2, dead_at)] {
            let report = Report {
                subtype,
                slot: 0,
                tick,
                digest: format!("{subtype}:{tick}"),
                forward: None,
            };
            assert!(
                model
                    .members
                    .get_mut(&1)
                    .unwrap()
                    .life
                    .apply(0, &report, now, None)
                    .accepted
            );
        }
        let report = Report {
            subtype: 21,
            slot: 1,
            tick: 1,
            digest: "survivor".into(),
            forward: None,
        };
        model
            .members
            .get_mut(&2)
            .unwrap()
            .life
            .apply(1, &report, Instant::now(), None);
        let (ta, ra) = mpsc::channel(8);
        let (tb, mut rb) = mpsc::channel(2);
        let shared = Arc::new(Mutex::new(Runtime {
            public_key: None,
            managed_round: None,
            relay: None,
            scope: Arc::new(()),
            model,
            players: synthetic_players(&packets),
            bodies: normalize_legacy(&packets),
            channels: HashMap::from([
                (
                    1,
                    Channel {
                        relay_authority: None,
                        endpoint: 0,
                        connection: 11,
                        sender: ta,
                    },
                ),
                (
                    2,
                    Channel {
                        relay_authority: None,
                        endpoint: 0,
                        connection: 22,
                        sender: tb,
                    },
                ),
            ]),
            last_visits: HashMap::new(),
            requested: HashSet::new(),
            rostered: HashSet::new(),
            title: b"RustRoom".to_vec(),
            selector: 1,
            ready: packet(241, vec![]),
            ips: HashMap::new(),
            ports: HashMap::new(),
            floor_catalog: pickups::Catalog::default(),
            rules: RoomRules::default(),
            abort: None,
        }));
        let mut session = Session {
            shared: shared.clone(),
            inbox: ra,
            identity: 1,
            connection: 11,
        };
        // Fixtures provide bytes, not authority to enable outcomes. Even with
        // a mature result and rank packets loaded, Disabled must stay Armed.
        session.tick().unwrap();
        assert_eq!(shared.lock().unwrap().model.phase, Phase::Armed { round });
        assert!(session.inbox.try_recv().is_err());
        assert!(rb.try_recv().is_err());
        shared.lock().unwrap().rules = RoomRules {
            result: ResultPolicy::Duel,
            departure: DeparturePolicy::Forfeit,
            life: rules::LifePolicy::DuelReports,
            ..RoomRules::default()
        };
        assert!(session.tick().is_err());
        assert_eq!(shared.lock().unwrap().model.phase, Phase::Armed { round });
        assert!(session.inbox.try_recv().is_err());
        assert!(rb.try_recv().is_err());
        // Replace only the failed test transport with enough capacity; the
        // same mature outcome must be deliverable once, without lost state.
        let (tb, rb) = mpsc::channel(8);
        shared.lock().unwrap().channels.insert(
            2,
            Channel {
                relay_authority: None,
                endpoint: 0,
                connection: 22,
                sender: tb,
            },
        );
        let mut other = Session {
            shared: shared.clone(),
            inbox: rb,
            identity: 2,
            connection: 22,
        };
        session.tick().unwrap();
        assert!(
            matches!(shared.lock().unwrap().model.phase,Phase::Ending{round:current,..}if current==round)
        );
        session.tick().unwrap();
        let mut a_batches = Vec::new();
        let mut b_batches = Vec::new();
        for _ in 0..3 {
            a_batches.push(session.inbox.try_recv().unwrap());
            b_batches.push(other.inbox.try_recv().unwrap());
        }
        assert!(session.inbox.try_recv().is_err());
        assert!(other.inbox.try_recv().is_err());
        assert_eq!(
            a_batches
                .iter()
                .map(|b| b.packets[0].1.opcode)
                .collect::<Vec<_>>(),
            [175, 180, 110]
        );
        assert_eq!(
            read_uint(&a_batches[1].packets[0].1.payload[1..2], Width::U8).unwrap(),
            1
        );
        assert!(session.valid(&a_batches[2]));
        session.finish(&a_batches[2]).unwrap();
        assert!(session.valid(&a_batches[2]));
        other.finish(&b_batches[2]).unwrap();
        assert_eq!(shared.lock().unwrap().model.phase, Phase::Idle);
        assert!(!session.valid(&a_batches[2]));
        {
            let mut r = shared.lock().unwrap();
            r.model.ready(b).unwrap();
            let next = r.model.begin(a).unwrap();
            assert_eq!(next, round + 1);
        }
        assert!(session.finish(&a_batches[2]).is_err());
        assert!(!session.valid(&a_batches[0]));
        // Test transport has no normal departure fixture. Avoid unrelated Drop
        // effects obscuring the original outcome assertions.
        shared.lock().unwrap().channels.clear();
    }
    #[test]
    fn recipient_queue_failure_publishes_neither_membership_nor_partial_batch() {
        let reply = Reply {
            header: [0; 37],
            opcode: 107,
            payload: vec![],
        };
        let (ta, mut ra) = mpsc::channel(2);
        let (tb, mut rb) = mpsc::channel(2);
        let mut runtime = Runtime {
            public_key: None,
            managed_round: None,
            relay: None,
            scope: Arc::new(()),
            model: Room::default(),
            players: HashMap::new(),
            bodies: HashMap::new(),
            channels: HashMap::from([
                (
                    1,
                    Channel {
                        relay_authority: None,
                        endpoint: 0,
                        connection: 11,
                        sender: ta,
                    },
                ),
                (
                    2,
                    Channel {
                        relay_authority: None,
                        endpoint: 0,
                        connection: 22,
                        sender: tb.clone(),
                    },
                ),
            ]),
            last_visits: HashMap::new(),
            requested: HashSet::new(),
            rostered: HashSet::new(),
            title: b"RustRoom".to_vec(),
            selector: 0,
            ready: reply.clone(),
            ips: HashMap::new(),
            ports: HashMap::new(),
            floor_catalog: pickups::Catalog::default(),
            rules: RoomRules::default(),
            abort: None,
        };
        let mut next = runtime.model.clone();
        let key = next.enter(1, 11, true).unwrap();
        let batch = || Runtime::member_batch(key, None, 0, vec![("test".into(), reply.clone())]);
        for _ in 0..2 {
            tb.try_send(batch())
                .unwrap_or_else(|_| panic!("fill bounded queue"));
        }
        assert!(
            runtime
                .publish(next.clone(), vec![(1, batch()), (2, batch())])
                .is_err()
        );
        assert!(runtime.model.members.is_empty());
        assert!(ra.try_recv().is_err());
        assert!(rb.try_recv().is_ok());
        assert!(rb.try_recv().is_ok());
        assert!(
            runtime
                .publish(next, vec![(1, batch()), (2, batch())])
                .is_ok()
        );
        assert!(runtime.model.current(key));
        assert!(ra.try_recv().is_ok());
        assert!(rb.try_recv().is_ok());

        // A successfully parsed claim must NOT be consumed when B's bounded
        // queue cannot be reserved. Retry then reaches BOTH recipients once.
        let mut active = runtime.model.clone();
        let peer = active.enter(2, 22, false).unwrap();
        active.setup(key).unwrap();
        active.setup(peer).unwrap();
        active.ready(peer).unwrap();
        let round = active.begin(key).unwrap();
        active.ack(key, round).unwrap();
        active.ack(peer, round).unwrap();
        active.pickups = pickups::Round::begin(round, &pickups::tests::catalog()).unwrap();
        runtime.model = active;
        runtime
            .bodies
            .insert(Template::Start121, reply.payload.clone());
        for _ in 0..2 {
            tb.try_send(Runtime::member_batch(peer, Some(round), 0, vec![]))
                .unwrap_or_else(|_| panic!("fill B"));
        }
        let shared = Arc::new(Mutex::new(runtime));
        let session = Session {
            shared: shared.clone(),
            inbox: ra,
            identity: 1,
            connection: 11,
        };
        let mut body = Vec::new();
        for (value, width) in [
            (0, kr_wire::Width::U8),
            (0, kr_wire::Width::U8),
            (4, kr_wire::Width::U8),
            (3, kr_wire::Width::U8),
            (17, kr_wire::Width::U32),
            (2, kr_wire::Width::U16),
            (23, kr_wire::Width::U32),
            (12346, kr_wire::Width::U32),
            (0, kr_wire::Width::U8),
        ] {
            body.extend(kr_wire::write_uint(value, width).unwrap());
        }
        assert!(session.packet(124, &body).is_err());
        assert!(rb.try_recv().is_ok());
        assert!(rb.try_recv().is_ok());
        let mut session = session;
        assert!(session.inbox.try_recv().is_err());
        assert!(session.packet(124, &body).unwrap().pickup.unwrap().accepted);
        let a = session.inbox.try_recv().unwrap();
        let b = rb.try_recv().unwrap();
        assert_eq!(a.packets[0].1.opcode, 165);
        assert_eq!(a.packets[0].1.payload, b.packets[0].1.payload);
        assert!(!session.packet(124, &body).unwrap().pickup.unwrap().accepted);
        assert!(session.inbox.try_recv().is_err());
        assert!(rb.try_recv().is_err());
        // Spawn and due158 each need one reserved slot per recipient. A failed
        // publication must leave ID1/serial1/capacity available for a real retry.
        shared.lock().unwrap().model.terrain = Some(crate::terrain::tests::map());
        shared
            .lock()
            .unwrap()
            .model
            .terrain
            .as_mut()
            .unwrap()
            .begin(round);
        let mut bomb_body = Vec::new();
        for (value, width) in [
            (12346, Width::U32),
            (0, Width::U8),
            (0, Width::U8),
            (1, Width::U32),
            (3, Width::U8),
            (3, Width::U8),
            (1, Width::U8),
            (0, Width::U8),
            (0, Width::U8),
            (0, Width::U8),
            (1, Width::U8),
            (1, Width::U32),
            (0, Width::U8),
        ] {
            bomb_body.extend(kr_wire::write_uint(value, width).unwrap());
        }
        for _ in 0..2 {
            tb.try_send(Runtime::member_batch(peer, Some(round), 0, vec![]))
                .unwrap_or_else(|_| panic!("fill B"));
        }
        bomb_body[11] = kr_wire::write_uint(4, Width::U8).unwrap()[0];
        assert!(session.packet(118, &bomb_body).is_err());
        assert!(session.inbox.try_recv().is_err());
        for _ in 0..2 {
            assert!(rb.try_recv().is_ok());
        }
        let accepted = session.packet(118, &bomb_body).unwrap().bomb.unwrap();
        assert!(accepted.accepted);
        assert_eq!(accepted.id, Some(1));
        let spawn_a = session.inbox.try_recv().unwrap();
        let due_a = session.inbox.try_recv().unwrap();
        let spawn_b = rb.try_recv().unwrap();
        let due_b = rb.try_recv().unwrap();
        assert_eq!(spawn_a.packets[0].1.opcode, 159);
        assert_eq!(spawn_a.packets[0].1.payload, spawn_b.packets[0].1.payload);
        assert_eq!(due_a.packets[0].1.opcode, 158);
        assert_eq!(due_a.packets[0].1.payload, due_b.packets[0].1.payload);
        assert!(due_a.due.duration_since(spawn_a.due) >= Duration::from_secs(6));
        assert!(session.valid(&due_a));
        let (ticket, recipient) = due_b.bomb_complete.as_ref().unwrap();
        shared
            .lock()
            .unwrap()
            .model
            .bombs
            .delivered(ticket, *recipient);
        assert!(session.valid(&due_a));
        assert_eq!(
            shared
                .lock()
                .unwrap()
                .model
                .terrain
                .as_ref()
                .unwrap()
                .clearance(round, 3, 5),
            None
        );
        session.finish(&due_a).unwrap();
        assert_eq!(
            shared
                .lock()
                .unwrap()
                .model
                .terrain
                .as_ref()
                .unwrap()
                .clearance(round, 3, 5),
            Some(1)
        );
        assert!(!session.valid(&due_a));
        // Counter does not reuse the old wire ID; leave invalidates every due job.
        bomb_body[6..10].copy_from_slice(&kr_wire::write_uint(2, Width::U32).unwrap());
        assert_eq!(
            session.packet(118, &bomb_body).unwrap().bomb.unwrap().id,
            Some(2)
        );
        session.inbox.try_recv().unwrap();
        let old_due = session.inbox.try_recv().unwrap();
        shared.lock().unwrap().model.leave(key).unwrap();
        assert!(!session.valid(&old_due));
        // This queue-only fixture has no native departure templates.
        shared.lock().unwrap().channels.clear();
    }
}

#[cfg(test)]
mod four_abort_tests {
    use super::*;
    fn runtime() -> (Shared, Vec<Session>, [Membership; 4], u64) {
        let mut model = Room::with_capacity(4).unwrap();
        let keys = std::array::from_fn(|i| {
            let k = model.enter(i as u32 + 1, i as u64 + 11, i == 0).unwrap();
            model.setup(k).unwrap();
            if i > 0 {
                model.ready(k).unwrap();
            }
            k
        });
        let round = model.begin(keys[0]).unwrap();
        for key in keys {
            model.ack(key, round).unwrap();
        }
        let mut channels = HashMap::new();
        let mut inboxes = Vec::new();
        let mut packets = HashMap::new();
        let packet = |opcode, payload| Reply {
            header: [0; 37],
            opcode,
            payload,
        };
        packets.insert("start".into(), packet(121, vec![]));
        for (i, key) in keys.iter().enumerate() {
            let mut body = vec![0; 204];
            body[1..5]
                .copy_from_slice(&kr_wire::write_uint(key.identity as u64, Width::U32).unwrap());
            body[12..16].copy_from_slice(&kr_wire::write_uint(0, Width::U32).unwrap());
            packets.insert(
                legacy_player_alias(key.identity).unwrap().into(),
                packet(106, body),
            );
            let (sender, inbox) = mpsc::channel(8);
            channels.insert(
                key.identity,
                Channel {
                    relay_authority: None,
                    endpoint: 0,
                    connection: key.connection,
                    sender,
                },
            );
            inboxes.push((i, inbox));
        }
        let shared = Arc::new(Mutex::new(Runtime {
            public_key: None,
            managed_round: None,
            relay: None,
            scope: Arc::new(()),
            model,
            players: synthetic_players(&packets),
            bodies: normalize_legacy(&packets),
            channels,
            last_visits: HashMap::new(),
            requested: keys.into_iter().collect(),
            rostered: HashSet::new(),
            title: b"AbortRoom".to_vec(),
            selector: 1,
            ready: packet(241, vec![]),
            ips: HashMap::new(),
            ports: HashMap::new(),
            floor_catalog: pickups::Catalog::default(),
            rules: RoomRules {
                bomb: rules::BombPolicy::OwnerReports,
                life: rules::LifePolicy::TeamReports,
                result: ResultPolicy::Teams,
                departure: DeparturePolicy::UnrankedReturn,
                ..RoomRules::default()
            },
            abort: None,
        }));
        let sessions = inboxes
            .into_iter()
            .map(|(i, inbox)| Session {
                shared: shared.clone(),
                inbox,
                identity: keys[i].identity,
                connection: keys[i].connection,
            })
            .collect();
        (shared, sessions, keys, round)
    }
    fn zero() -> Vec<u8> {
        kr_wire::write_uint(0, Width::U32).unwrap()
    }
    #[test]
    fn unranked_recovery_requires_three_post_write_setups_then_all_refresh_completions() {
        let (shared, mut sessions, keys, round) = runtime();
        let queued = Runtime::member_batch(keys[1], Some(round), 6000, vec![]);
        {
            let mut runtime = shared.lock().unwrap();
            let next = runtime.model.clone();
            runtime
                .publish(next, vec![(keys[1].identity, queued)])
                .unwrap();
        }
        let old = sessions[1].inbox.try_recv().unwrap();
        assert!(sessions[1].valid(&old));
        shared.lock().unwrap().detach(1, keys[0].connection);
        assert!(!sessions[1].valid(&old));
        let mut returns = Vec::new();
        for s in &mut sessions[1..] {
            let b = s.inbox.try_recv().unwrap();
            assert_eq!(
                b.packets.iter().map(|p| p.1.opcode).collect::<Vec<_>>(),
                vec![180]
            );
            assert!(s.valid(&b));
            assert!(s.packet(585, &zero()).is_err()); // cannot set up before actual write
            s.finish(&b).unwrap();
            assert!(s.finish(&b).is_err());
            returns.push(b);
        }
        for s in &sessions[1..3] {
            s.packet(585, &zero()).unwrap();
        }
        sessions[1].packet(585, &zero()).unwrap();
        sessions[1].tick().unwrap();
        for s in &mut sessions[1..] {
            assert!(s.inbox.try_recv().is_err());
        }
        sessions[3].packet(585, &zero()).unwrap();
        sessions[1].tick().unwrap();
        let mut refresh = Vec::new();
        for s in &mut sessions[1..] {
            let b = s.inbox.try_recv().unwrap();
            assert_eq!(
                b.packets.iter().map(|p| p.1.opcode).collect::<Vec<_>>(),
                vec![107, 110, 110, 110]
            );
            assert!(s.valid(&b));
            refresh.push(b);
        }
        sessions[1].tick().unwrap();
        for s in &mut sessions[1..] {
            assert!(s.inbox.try_recv().is_err());
        }
        for (s, b) in sessions[1..3].iter().zip(&refresh[..2]) {
            s.finish(b).unwrap();
        }
        assert!(matches!(
            shared.lock().unwrap().model.phase,
            Phase::Ending { .. }
        ));
        sessions[3].finish(&refresh[2]).unwrap();
        let mut r = shared.lock().unwrap();
        assert_eq!(r.model.phase, Phase::Idle);
        assert_eq!(r.model.host, Some(2));
        assert!(r.abort.as_ref().unwrap().completed);
        assert!(r.model.enter(1, 900, false).is_err()); // sealed, no replacement/restart
        assert!(r.model.begin(keys[1]).is_err());
        assert!(!r.model.deliverable(keys[1], Some(round)));
        r.channels.clear();
        drop(r);
        assert!(!sessions[1].valid(&returns[0]));
    }
    #[test]
    fn abort_refresh_queue_failure_does_not_partially_publish_or_consume_barrier() {
        let (shared, mut sessions, keys, _) = runtime();
        shared.lock().unwrap().detach(4, keys[3].connection);
        for s in &mut sessions[..3] {
            let b = s.inbox.try_recv().unwrap();
            s.finish(&b).unwrap();
            s.packet(585, &zero()).unwrap();
        }
        // Fill exactly one recipient; other two must receive no partial cleanup.
        let mut r = shared.lock().unwrap();
        for _ in 0..8 {
            let b = Runtime::member_batch(keys[2], None, 0, vec![]);
            r.channels[&3]
                .sender
                .try_send(b)
                .unwrap_or_else(|_| panic!("fill queue"));
        }
        assert!(r.abort_refresh().is_err());
        assert!(!r.abort.as_ref().unwrap().refresh_queued);
        drop(r);
        assert!(sessions[0].inbox.try_recv().is_err());
        assert!(sessions[1].inbox.try_recv().is_err());
        for _ in 0..8 {
            sessions[2].inbox.try_recv().unwrap();
        }
        sessions[0].tick().unwrap();
        for s in &mut sessions[..3] {
            assert_eq!(s.inbox.try_recv().unwrap().packets.len(), 4);
        }
        shared.lock().unwrap().channels.clear();
    }
    #[test]
    fn second_loss_during_unranked_recovery_invalidates_all_pending_work() {
        let (shared, mut sessions, keys, _) = runtime();
        shared.lock().unwrap().detach(4, keys[3].connection);
        let pending = sessions[0].inbox.try_recv().unwrap();
        assert!(sessions[0].valid(&pending));
        shared.lock().unwrap().detach(2, keys[1].connection);
        assert!(!sessions[0].valid(&pending));
        let r = shared.lock().unwrap();
        assert!(r.channels.is_empty());
        assert_eq!(r.abort.as_ref().unwrap().departed.key, keys[3]);
        assert!(!r.abort.as_ref().unwrap().completed);
    }
}

#[cfg(test)]
mod eight_baseline_tests {
    use super::*;
    use crate::life::Report;
    fn armed() -> (Room, [Membership; 8], u64) {
        let mut room = Room::with_capacity(8).unwrap();
        let keys = std::array::from_fn(|i| {
            let k = room.enter(i as u32 + 1, i as u64 + 31, i == 0).unwrap();
            room.setup(k).unwrap();
            if i > 0 {
                room.ready(k).unwrap();
            }
            k
        });
        let round = room.begin(keys[0]).unwrap();
        for k in keys {
            room.ack(k, round).unwrap();
        }
        (room, keys, round)
    }
    fn report(slot: u8, subtype: u8, tick: u32) -> Report {
        Report {
            slot,
            subtype,
            tick,
            digest: format!("{slot}:{subtype}:{tick}"),
            forward: None,
        }
    }
    #[test]
    fn eight_native_accepted_cp210_rank_hashes_are_wire_regression_only() {
        for (team, hash) in [
            (
                0,
                "3ce601dd5323d5a0f5fae33c932da1f92f4129c2d33b3a681260eb961f1a33a5",
            ),
            (
                1,
                "5efff3e6576d7241a58b889e27b1f085e66c9313ecfd6a74ade4c630469551bf",
            ),
        ] {
            let body = crate::four_results::rank_body(&[0, 1, 0, 1, 0, 1, 0, 1], team).unwrap();
            assert_eq!(body.len(), 1082);
            assert_eq!(crate::sha256_hex(&body), hash);
        }
        assert!(crate::four_results::rank_body(&[0; 7], 0).is_err());
    }
    #[test]
    fn eight_waits_all_seven_distinct_life_writes_then_current_witness_and_fences_loss() {
        let (mut room, keys, round) = armed();
        let at = Instant::now();
        for slot in [0, 2, 4, 6] {
            let life = &mut room.members.get_mut(&(slot as u32 + 1)).unwrap().life;
            assert!(life.apply(slot, &report(slot, 17, 1), at, None).accepted);
            assert!(life.apply(slot, &report(slot, 19, 2), at, None).accepted);
        }
        let witness = &mut room.members.get_mut(&2).unwrap().life;
        assert!(
            witness
                .apply(1, &report(1, 21, 1), at + Duration::from_secs(1), None)
                .accepted
        );
        let targets = keys[1..].to_vec();
        room.members
            .get_mut(&1)
            .unwrap()
            .life
            .forward_report_targets("death", &targets)
            .unwrap();
        for target in &targets[..6] {
            room.members
                .get_mut(&1)
                .unwrap()
                .life
                .report_delivered("death", *target);
            assert_eq!(
                crate::four_results::winner(&room, at + Duration::from_secs(8)),
                None
            );
        }
        room.members
            .get_mut(&1)
            .unwrap()
            .life
            .report_delivered("death", targets[5]);
        assert_eq!(
            crate::four_results::winner(&room, at + Duration::from_secs(8)),
            None
        );
        room.members
            .get_mut(&1)
            .unwrap()
            .life
            .report_delivered("death", targets[6]);
        assert_eq!(
            crate::four_results::winner(&room, at + Duration::from_secs(8)),
            Some(1)
        );
        room.fence_eight(keys[7]).unwrap();
        assert!(!room.deliverable(keys[0], Some(round)));
        assert!(room.enter(8, 999, false).is_err());
        assert_eq!(room.host, Some(1));
    }
    #[test]
    fn eight_barrier_needs_eighth_ack_and_measured_bomb_capacities() {
        let mut room = Room::with_capacity(8).unwrap();
        let keys: [Membership; 8] = std::array::from_fn(|i| {
            let k = room.enter(i as u32 + 1, i as u64 + 41, i == 0).unwrap();
            room.setup(k).unwrap();
            if i > 0 {
                room.ready(k).unwrap();
            }
            k
        });
        assert!(room.enter(9, 77, false).is_err());
        let round = room.begin(keys[0]).unwrap();
        for k in &keys[..7] {
            assert!(!room.ack(*k, round).unwrap());
        }
        assert!(room.ack(keys[7], round).unwrap());
        assert_eq!(room.bomb_capacity, [1, 2, 1, 1, 2, 1, 1, 1]);
        let endpoints: Vec<_> = (0..8).map(|i| (0x0a4d0002 + i, 32001 + i as u16)).collect();
        assert_eq!(
            wire::peer153(&[0; 61], 12346, 7, &endpoints).unwrap().len(),
            205
        );
    }
}

#[cfg(test)]
mod ninth_probe_tests {
    use super::*;
    fn full() -> Session {
        let mut model = Room::with_capacity(8).unwrap();
        for i in 1..=8 {
            let key = model.enter(i, u64::from(i) + 10, i == 1).unwrap();
            model.setup(key).unwrap();
            if i > 1 {
                model.ready(key).unwrap();
            }
        }
        let (sender, inbox) = mpsc::channel(2);
        let shared = Arc::new(Mutex::new(Runtime {
            public_key: None,
            managed_round: None,
            relay: None,
            scope: Arc::new(()),
            model,
            players: HashMap::new(),
            bodies: HashMap::from([(Template::Join87, vec![0; 7])]),
            channels: HashMap::from([(
                9,
                Channel {
                    relay_authority: None,
                    endpoint: 0,
                    connection: 99,
                    sender,
                },
            )]),
            last_visits: HashMap::new(),
            requested: HashSet::new(),
            rostered: HashSet::new(),
            title: b"RustRoom".to_vec(),
            selector: 1,
            ready: Reply {
                header: [0; 37],
                opcode: 241,
                payload: vec![],
            },
            ips: HashMap::new(),
            ports: HashMap::new(),
            floor_catalog: pickups::Catalog::default(),
            rules: RoomRules {
                bomb: rules::BombPolicy::OwnerReports,
                life: rules::LifePolicy::TeamReports,
                result: ResultPolicy::Teams,
                departure: DeparturePolicy::FenceActive,
                teams: TeamPolicy::BalancedManner,
                roster: rules::RoundRoster::Full,
                ninth_admission_probe: true,
                slot_allocation: rules::SlotAllocation::LegacyIdentity,
                endpoint: rules::EndpointPolicy::LegacyDirect,
            },
            abort: None,
        }));
        Session {
            shared,
            inbox,
            identity: 9,
            connection: 99,
        }
    }
    fn join() -> Vec<u8> {
        [
            (1, Width::U16),
            (1, Width::U32),
            (0, Width::U16),
            (1, Width::U8),
            (5, Width::U16),
        ]
        .into_iter()
        .flat_map(|(value, width)| kr_wire::write_uint(value, width).unwrap())
        .collect()
    }
    #[test]
    fn full_probe_error_has_no_membership_slot_round_or_other_recipient_effects() {
        let mut session = full();
        let original: Vec<_> = session
            .shared
            .lock()
            .unwrap()
            .model
            .members
            .values()
            .map(|m| (m.key, m.slot, m.team, m.ready, m.setup))
            .collect();
        for _ in 0..2 {
            let observation = session.packet(82, &join()).unwrap();
            assert_eq!(
                observation.admission_rejection,
                Some("diagnostic_eight_room_full")
            );
            let batch = session.inbox.try_recv().unwrap();
            assert!(session.valid(&batch));
            assert!(matches!(batch.guard, Guard::Lobby { identity: 9, .. }));
            assert_eq!(batch.packets.len(), 1);
            assert_eq!(batch.packets[0].1.opcode, 87);
            assert_eq!(
                read_uint(&batch.packets[0].1.payload, Width::U8).unwrap(),
                2
            );
            let r = session.shared.lock().unwrap();
            assert_eq!(r.model.phase, Phase::Idle);
            assert!(!r.model.members.contains_key(&9));
            assert!(!r.last_visits.contains_key(&9));
            assert!(!r.requested.iter().any(|m| m.identity == 9));
            assert_eq!(
                r.model
                    .members
                    .values()
                    .map(|m| (m.key, m.slot, m.team, m.ready, m.setup))
                    .collect::<Vec<_>>(),
                original
            );
        }
        // The negative path must not consume the next round generation.
        let mut r = session.shared.lock().unwrap();
        let host = r.model.members[&1].key;
        assert_eq!(r.model.begin(host).unwrap(), 1);
    }
    #[test]
    fn queue_failure_and_wrong_context_cannot_mutate_the_full_room() {
        let session = full();
        assert!(session.packet(82, &[]).is_err());
        session.packet(82, &join()).unwrap();
        session.packet(82, &join()).unwrap();
        assert!(session.packet(82, &join()).is_err());
        let r = session.shared.lock().unwrap();
        assert_eq!(r.model.phase, Phase::Idle);
        assert_eq!(r.model.members.len(), 8);
        assert!(!r.model.members.contains_key(&9));
        assert!(!r.last_visits.contains_key(&9));
    }
}

#[cfg(test)]
mod registry_foundation_tests {
    use super::*;
    use crate::room_registry::{Registry, RoomId, RoomKey};
    fn empty(marker: u8) -> Shared {
        Arc::new(Mutex::new(Runtime {
            public_key: None,
            managed_round: None,
            relay: None,
            scope: Arc::new(()),
            model: Room::default(),
            players: HashMap::new(),
            bodies: HashMap::from([(Template::Table238, vec![marker])]),
            channels: HashMap::new(),
            last_visits: HashMap::new(),
            requested: HashSet::new(),
            rostered: HashSet::new(),
            title: b"ProbeRoom".to_vec(),
            selector: 0,
            ready: packet(241, vec![]),
            ips: HashMap::new(),
            ports: HashMap::new(),
            floor_catalog: pickups::Catalog::default(),
            rules: RoomRules::default(),
            abort: None,
        }))
    }
    #[test]
    fn managed_team_config_rejects_unmeasured_modes_limits_and_teams_before_serving() {
        let prototype = empty(0x77);
        let mut stream = vec![0; 54 + 26 * 8];
        stream[0] = 24;
        let size = stream.len();
        stream[1..5].copy_from_slice(&((size - 5) as u32).to_le_bytes());
        stream[49] = 8;
        for slot in 0..8 {
            let at = 54 + 26 * slot;
            stream[at] = slot as u8;
            stream[at + 2] = slot as u8;
            stream[at + 3..at + 5].copy_from_slice(&(slot as u16).to_le_bytes());
            stream[at + 5] = (slot % 2) as u8;
        }
        let mut start = kr_wire::write_uint(0, Width::U8).unwrap();
        start.extend(kr_wire::write_uint(size as u64, Width::U32).unwrap());
        let mut pos = 0;
        while pos < size {
            let (n, width) = if size - pos >= 4 {
                (4, Width::U32)
            } else if size - pos >= 2 {
                (2, Width::U16)
            } else {
                (1, Width::U8)
            };
            let mut raw = [0; 8];
            raw[..n].copy_from_slice(&stream[pos..pos + n]);
            start.extend(kr_wire::write_uint(u64::from_le_bytes(raw), width).unwrap());
            pos += n;
        }
        start.extend([0; 18]);
        {
            let mut source = prototype.lock().unwrap();
            source.model = Room::with_allocation(8, rules::SlotAllocation::FirstVacant).unwrap();
            source.bodies.insert(Template::Start121, start);
        }
        let mut cfg = ManagedRoundConfig {
            mode: ManagedRoundMode::Teams,
            teams: vec![0, 1, 0, 1],
            battle: None,
            initial_bomb_capacity: vec![1, 2, 1, 1],
        };
        let accepted = load_managed_round(&cfg, &prototype).unwrap();
        assert_eq!(accepted.rules.roster, rules::RoundRoster::Contiguous(4));
        assert_eq!(accepted.rules.departure, DeparturePolicy::FenceActive);
        assert_eq!(accepted.initial_bomb_capacity, [1, 2, 1, 1]);
        cfg.initial_bomb_capacity[1] = 1;
        assert!(load_managed_round(&cfg, &prototype).is_err());
        cfg.initial_bomb_capacity[1] = 2;
        cfg.teams[1] = 0;
        assert!(load_managed_round(&cfg, &prototype).is_err());
        cfg.teams[1] = 1;
        cfg.mode = ManagedRoundMode::Duel;
        assert!(load_managed_round(&cfg, &prototype).is_err());
        cfg.mode = ManagedRoundMode::Teams;
        cfg.battle = Some(BattleConfig {
            mode: "post_blast_untrapped_drained".into(),
            packets: HashMap::new(),
        });
        assert!(load_managed_round(&cfg, &prototype).is_err());
        cfg.battle = None;
        for count in [0, 1, 3, 5, 9] {
            cfg.teams = (0..count).map(|slot| (slot % 2) as u8).collect();
            cfg.initial_bomb_capacity = rules::default_bomb_capacity(count);
            assert!(load_managed_round(&cfg, &prototype).is_err());
        }
        assert!(prototype.lock().unwrap().model.members.is_empty());
        assert_eq!(prototype.lock().unwrap().model.phase, Phase::Idle);
    }
    #[test]
    fn registry_routes_actual_session_queues_without_cross_room_recipients() {
        let mut rooms = Registry::default();
        let alpha = rooms.insert(1, RoomId(101), empty(0xa1)).unwrap();
        let beta = rooms.insert(2, RoomId(202), empty(0xb2)).unwrap();
        let selected = rooms.selection(1, RoomId(101)).unwrap();
        let mut a = Session::open(rooms.get(selected).unwrap(), 1, 11).unwrap();
        let mut b = Session::open(rooms.get(beta).unwrap(), 2, 22).unwrap();
        a.packet(528, &[]).unwrap();
        let first = a.inbox.try_recv().unwrap();
        assert!(a.valid(&first));
        assert_eq!(first.packets[0].1.opcode, 238);
        assert_eq!(first.packets[0].1.payload, vec![0xa1]);
        assert!(b.inbox.try_recv().is_err());
        b.packet(528, &[]).unwrap();
        let second = b.inbox.try_recv().unwrap();
        assert!(b.valid(&second));
        assert_eq!(second.packets[0].1.payload, vec![0xb2]);
        assert!(a.inbox.try_recv().is_err());
        assert!(rooms.selection(1, RoomId(202)).is_err());
        assert!(
            rooms
                .get(RoomKey {
                    generation: beta.generation + 1,
                    ..beta
                })
                .is_err()
        );
        assert!(a.inbox.try_recv().is_err() && b.inbox.try_recv().is_err());
        drop(a);
        assert!(
            rooms
                .get(alpha)
                .unwrap()
                .lock()
                .unwrap()
                .channels
                .is_empty()
        );
        assert_eq!(rooms.get(beta).unwrap().lock().unwrap().channels.len(), 1);
        b.packet(528, &[]).unwrap();
        assert_eq!(b.inbox.try_recv().unwrap().packets[0].1.payload, vec![0xb2]);
    }
    #[test]
    fn native_duel_burst167_is_bounded_observation_without_ack_or_world_effect() {
        let shared = empty(0x71);
        let mut session = Session::open(&shared, 70000, 11).unwrap();
        let peer = Session::open(&shared, 42, 22).unwrap();
        let round = {
            let mut runtime = shared.lock().unwrap();
            runtime.public_key = Some(RoomKey {
                session: 1,
                id: RoomId(7),
                generation: 1,
            });
            runtime.managed_round = Some(Arc::new(ManagedRound {
                bodies: HashMap::new(),
                initial_bomb_capacity: vec![1, 2],
                rules: RoomRules::default(),
                limits: rules::ReportLimits::measured(2).unwrap(),
            }));
            runtime.rules.roster = rules::RoundRoster::Contiguous(2);
            runtime.model = Room::with_allocation(4, rules::SlotAllocation::FirstVacant).unwrap();
            runtime.model.round_roster = rules::RoundRoster::Contiguous(2);
            let a = runtime.model.enter(70000, 11, true).unwrap();
            let b = runtime.model.enter(42, 22, false).unwrap();
            runtime.model.setup(a).unwrap();
            runtime.model.setup(b).unwrap();
            runtime.model.ready(b).unwrap();
            let round = runtime.model.begin(a).unwrap();
            runtime.model.ack(a, round).unwrap();
            runtime.model.ack(b, round).unwrap();
            round
        };
        assert!(session.packet(167, &[0; 21]).is_err());
        for index in 0..64 {
            let body = vec![0; [12, 22, 42][index % 3]];
            session.packet(167, &body).unwrap();
        }
        assert!(session.packet(167, &[0; 42]).is_err());
        assert_eq!(shared.lock().unwrap().model.phase, Phase::Armed { round });
        assert!(!shared.lock().unwrap().model.bombs.pending());
        assert!(session.inbox.try_recv().is_err());
        assert!(peer.inbox.is_empty());
        // Avoid exercising unrelated departure fixture construction on drop.
        shared.lock().unwrap().channels.clear();
    }
    #[test]
    fn full_actual_recipient_queue_in_one_room_does_not_charge_other_room_queue() {
        let mut rooms = Registry::default();
        let alpha = rooms.insert(1, RoomId(101), empty(0xa1)).unwrap();
        let beta = rooms.insert(2, RoomId(202), empty(0xb2)).unwrap();
        let mut a = Session::open(rooms.get(alpha).unwrap(), 1, 11).unwrap();
        let mut b = Session::open(rooms.get(beta).unwrap(), 2, 22).unwrap();
        let capacity = rules::recipient_queue(2);
        for _ in 0..capacity {
            a.packet(528, &[]).unwrap();
        }
        assert!(a.packet(528, &[]).is_err());
        assert_eq!(a.inbox.len(), capacity);
        assert!(b.inbox.try_recv().is_err());
        b.packet(528, &[]).unwrap();
        assert_eq!(b.inbox.try_recv().unwrap().packets[0].1.payload, vec![0xb2]);
        for _ in 0..capacity {
            assert_eq!(a.inbox.try_recv().unwrap().packets[0].1.payload, vec![0xa1]);
        }
        assert!(a.inbox.try_recv().is_err());
        a.packet(528, &[]).unwrap();
        assert_eq!(a.inbox.try_recv().unwrap().packets[0].1.payload, vec![0xa1]);
    }
    #[test]
    fn registry_retirement_rejects_live_channel_and_keeps_queued_recipient_valid() {
        let mut rooms = Registry::default();
        let alpha = rooms.insert(1, RoomId(101), empty(0xa1)).unwrap();
        let beta = rooms.insert(2, RoomId(202), empty(0xb2)).unwrap();
        let mut a = Session::open(rooms.get(alpha).unwrap(), 1, 11).unwrap();
        let mut b = Session::open(rooms.get(beta).unwrap(), 2, 22).unwrap();
        a.packet(528, &[]).unwrap();
        let can_retire = |shared: &Shared| {
            let room = shared.lock().unwrap();
            room.channels.is_empty()
                && room.model.members.is_empty()
                && room.model.phase == Phase::Idle
                && !room.model.bombs.pending()
        };
        assert!(rooms.remove_if(alpha, can_retire).is_err());
        let queued = a.inbox.try_recv().unwrap();
        assert!(a.valid(&queued));
        assert_eq!(queued.packets[0].1.payload, vec![0xa1]);
        assert_eq!(rooms.selection(1, RoomId(101)).unwrap(), alpha);
        drop(a);
        let retired = rooms.remove_if(alpha, can_retire).unwrap();
        assert!(retired.lock().unwrap().channels.is_empty());
        assert!(rooms.get(alpha).is_err());
        assert!(rooms.selection(1, RoomId(101)).is_err());
        assert!(rooms.insert(1, RoomId(101), empty(0xa3)).is_err());
        let replacement = rooms.insert(1, RoomId(303), empty(0xa3)).unwrap();
        assert!(replacement.generation > alpha.generation);
        assert!(rooms.get(alpha).is_err());
        let mut replacement_session =
            Session::open(rooms.get(replacement).unwrap(), 1, 33).unwrap();
        replacement_session.packet(528, &[]).unwrap();
        assert_eq!(
            replacement_session.inbox.try_recv().unwrap().packets[0]
                .1
                .payload,
            vec![0xa3]
        );
        assert!(!replacement_session.valid(&queued));
        assert!(b.inbox.try_recv().is_err());
        b.packet(528, &[]).unwrap();
        assert_eq!(b.inbox.try_recv().unwrap().packets[0].1.payload, vec![0xb2]);
    }
    #[test]
    fn registry_listing_uses_current_room_titles_and_keys_with_no_cross_room_state() {
        let mut rooms = Registry::default();
        let alpha = empty(0xa1);
        let beta = empty(0xb2);
        let template = wire::listing91_from_rows(&[wire::ListingRow {
            session: 1,
            room_id: 1,
            title: b"Template".to_vec(),
            opaque: [0x17; 22],
        }])
        .unwrap();
        for shared in [&alpha, &beta] {
            shared
                .lock()
                .unwrap()
                .bodies
                .insert(Template::Listing91, template.clone());
        }
        alpha.lock().unwrap().title = b"RoomAlpha".to_vec();
        beta.lock().unwrap().title = b"RoomBeta".to_vec();
        let a = rooms.insert(1, RoomId(101), alpha.clone()).unwrap();
        let b = rooms.insert(2, RoomId(202), beta.clone()).unwrap();
        let rows = wire::listing91_parse_rows(&rooms.listing91().unwrap()).unwrap();
        assert_eq!(
            (rows[0].session, rows[0].room_id, rows[0].title.as_slice()),
            (1, 101, b"RoomAlpha".as_slice())
        );
        assert_eq!(
            (rows[1].session, rows[1].room_id, rows[1].title.as_slice()),
            (2, 202, b"RoomBeta".as_slice())
        );
        assert_eq!(rows[0].opaque, [0x17; 22]);
        assert_eq!(rows[1].opaque, [0x17; 22]);
        beta.lock().unwrap().title = b"RoomGamma".to_vec();
        let updated = wire::listing91_parse_rows(&rooms.listing91().unwrap()).unwrap();
        assert_eq!(updated[0], rows[0]);
        assert_eq!(updated[1].title, b"RoomGamma");
        rooms
            .remove_if(b, |shared| shared.lock().unwrap().channels.is_empty())
            .unwrap();
        let remaining = wire::listing91_parse_rows(&rooms.listing91().unwrap()).unwrap();
        assert_eq!(remaining, vec![rows[0].clone()]);
        assert_eq!(rooms.selection(1, RoomId(101)).unwrap(), a);
        rooms
            .remove_if(a, |shared| shared.lock().unwrap().channels.is_empty())
            .unwrap();
        assert!(
            wire::listing91_parse_rows(&rooms.listing91().unwrap())
                .unwrap()
                .is_empty()
        );
    }
    #[test]
    fn managed_forks_have_fresh_scopes_visits_and_vacant_slots() {
        let prototype = empty(0x77);
        let a_key = RoomKey {
            session: 1,
            id: RoomId(1),
            generation: 1,
        };
        let b_key = RoomKey {
            session: 2,
            id: RoomId(2),
            generation: 2,
        };
        let epochs = Arc::new(crate::round_epochs::RoundEpochs::default());
        let a = fork(&prototype, a_key, epochs.clone(), None).unwrap();
        let b = fork(&prototype, b_key, epochs.clone(), None).unwrap();
        let mut old = Session::open(&a, 3, 33).unwrap();
        old.packet(528, &[]).unwrap();
        let batch = old.inbox.try_recv().unwrap();
        assert!(old.valid(&batch));
        assert!(!removable(&a));
        assert!(fork(&a, b_key, epochs, None).is_err());
        let new = Session::open(&b, 3, 33).unwrap();
        assert!(!new.valid(&batch));
        assert!(new.packet(104, &[0; 20]).is_err());
        {
            let mut r = b.lock().unwrap();
            let member = r.model.enter(3, 33, true).unwrap();
            assert_eq!(r.model.members[&3].slot, 0);
            assert!(r.model.current(member));
        }
        assert!(new.packet(96, &[0; 6]).is_ok());
        assert!(new.packet(266, &[0; 12]).is_ok());
        assert!(new.packet(96, &[0; 5]).is_err());
        assert!(new.packet(266, &[0; 13]).is_err());
        assert!(new.packet(104, &[0; 20]).is_err());
        drop(new);
        assert!(removable(&b));
        drop(old);
        assert!(removable(&a));
        assert!(removable(&prototype));
        assert!(!Arc::ptr_eq(
            &a.lock().unwrap().scope,
            &b.lock().unwrap().scope
        ));
    }
}
