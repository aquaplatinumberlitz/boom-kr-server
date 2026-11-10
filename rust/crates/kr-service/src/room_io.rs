//! Fixed-room adapters with bounded recipient queues and membership fences.
use super::{ActorProfile, LobbyPacketPin, Reply};
use crate::bombs;
use crate::{
    pickups,
    room::{Member, Membership, Phase, Room},
    room_wire,
};
use kr_wire::{Width, decode, read_uint};
use serde::Deserialize;
use std::{
    collections::{HashMap, HashSet},
    io,
    net::Ipv4Addr,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::sync::mpsc;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub title: String,
    pub packets: HashMap<String, LobbyPacketPin>,
    #[serde(default)]
    pub terrain: Option<crate::terrain::Config>,
    #[serde(default)]
    pub battle: Option<BattleConfig>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BattleConfig {
    pub mode: String,
    pub packets: HashMap<String, LobbyPacketPin>,
}
pub type Shared = Arc<Mutex<Runtime>>;
struct Channel {
    connection: u64,
    sender: mpsc::Sender<Batch>,
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
    pub guard: Guard,
    pub due: Instant,
    pub packets: Vec<(String, Reply)>,
    pub setup: Option<Membership>,
    pub subject: Option<Membership>,
    pub bomb_complete: Option<(bombs::Ticket, Membership)>,
    pub life_complete: Option<(Membership, u64, String)>,
    pub outcome_complete: Option<(Membership, u64)>,
}
pub struct Runtime {
    model: Room,
    packets: HashMap<String, Reply>,
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
pub fn load(cfg: &Config, actors: &HashMap<Ipv4Addr, ActorProfile>) -> io::Result<Shared> {
    if cfg.title.is_empty()
        || cfg.title.len() > 32
        || !cfg.title.bytes().all(|b| (32..=126).contains(&b))
    {
        return Err(err("fixed room title must be bounded ASCII"));
    }
    let expected = [
        ("create", 86, 11),
        ("join", 87, 7),
        ("player_A", 106, 204),
        ("player_B", 106, 204),
        ("listing", 91, 0),
        ("table", 238, 72),
        ("map", 117, 8),
        ("floor", 582, 46),
        ("start", 121, 129),
        ("go", 197, 14),
        ("play", 196, 7),
        ("peer", 153, 97),
    ];
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
    if room_wire::listing_title(&packets["listing"].payload)? != cfg.title.as_bytes() {
        return Err(err("fixed listing title mismatch"));
    }
    for id in [1, 2] {
        let alias = if id == 1 { "player_A" } else { "player_B" };
        if read_uint(&packets[alias].payload[1..5], Width::U32).map_err(io::Error::other)? != id {
            return Err(err("room player identity mismatch"));
        }
    }
    let ready = actors
        .values()
        .next()
        .ok_or_else(|| err("missing actors"))?
        .packets["ready"]
        .clone();
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
            if op == 180 {
                room_wire::room_selector(&packet.payload, 180, 0)?;
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
    let mut model = Room::default();
    model.terrain = terrain;
    Ok(Arc::new(Mutex::new(Runtime {
        model,
        packets,
        channels: HashMap::new(),
        last_visits: HashMap::new(),
        requested: HashSet::new(),
        rostered: HashSet::new(),
        title: cfg.title.as_bytes().to_vec(),
        selector: 0,
        ready,
        ips: actors
            .iter()
            .map(|(ip, a)| (a.identity, u32::from(*ip)))
            .collect(),
        ports: HashMap::new(),
        floor_catalog: pickups::Catalog::from_floor(&packets_floor_body)?,
    })))
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
    fn reply(&self, alias: &str, opcode: Option<u32>, body: Option<Vec<u8>>) -> Reply {
        let mut p = self.packets[alias].clone();
        if let Some(op) = opcode {
            p.opcode = op;
        }
        if let Some(b) = body {
            p.payload = b;
        }
        p
    }
    fn player(&self, m: &Member, host: Option<u32>) -> io::Result<Reply> {
        let alias = if m.key.identity == 1 {
            "player_A"
        } else {
            "player_B"
        };
        Ok(self.reply(
            alias,
            None,
            Some(room_wire::player(
                &self.packets[alias].payload,
                m,
                host == Some(m.key.identity),
            )?),
        ))
    }
    fn slot(&self, m: &Member, host: Option<u32>) -> io::Result<Reply> {
        let alias = if m.key.identity == 1 {
            "player_A"
        } else {
            "player_B"
        };
        Ok(self.reply(
            alias,
            Some(110),
            Some(room_wire::slot(
                &self.packets[alias].payload,
                m,
                host == Some(m.key.identity),
            )?),
        ))
    }
    fn member_batch(
        key: Membership,
        round: Option<u64>,
        delay: u64,
        packets: Vec<(String, Reply)>,
    ) -> Batch {
        Batch {
            guard: Guard::Member(key, round),
            due: Instant::now() + Duration::from_millis(delay),
            packets,
            setup: None,
            subject: None,
            bomb_complete: None,
            life_complete: None,
            outcome_complete: None,
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
        }
    }
    // Reserve every recipient queue entry before publishing any state or batch.
    // Physical TCP delivery can still fail; barrier/recipient fences remain required.
    fn publish(&mut self, next: Room, effects: Vec<(u32, Batch)>) -> io::Result<()> {
        let mut reserved = Vec::new();
        for (id, batch) in effects {
            let channel = self
                .channels
                .get(&id)
                .ok_or_else(|| err("missing recipient channel"))?;
            let permit = channel
                .sender
                .clone()
                .try_reserve_owned()
                .map_err(|_| err("room recipient queue unavailable"))?;
            reserved.push((permit, batch));
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
        let remove = self.reply("player_A", Some(107), Some(room_wire::removal(departed)?));
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
        let removal = self.reply("player_A", Some(107), Some(room_wire::removal(departed)?));
        let alias = if departed.slot == 0 {
            "rank"
        } else {
            "rank_Awin"
        };
        let returning = self.reply(
            "return",
            None,
            Some(room_wire::room_selector(
                &self.packets["return"].payload,
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
                    vec![("forfeit175".into(), self.reply(alias, None, None))],
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
    fn detach(&mut self, id: u32, connection: u64) {
        if self
            .channels
            .get(&id)
            .is_none_or(|c| c.connection != connection)
        {
            return;
        }
        if let Ok(m) = self.member(id, connection) {
            let mut next = self.model.clone();
            let closing_active =
                self.packets.contains_key("rank") && matches!(next.phase, Phase::Armed { .. });
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
    pub fn open(shared: &Shared, identity: u32, connection: u64) -> io::Result<Self> {
        let mut runtime = shared.lock().map_err(|_| err("room unavailable"))?;
        if runtime.channels.contains_key(&identity) {
            return Err(err("duplicate room channel"));
        }
        let (sender, inbox) = mpsc::channel(8);
        runtime
            .channels
            .insert(identity, Channel { connection, sender });
        if runtime.model.host.is_some() {
            let batch = runtime.lobby_batch(
                identity,
                connection,
                0,
                vec![("listing91".into(), runtime.reply("listing", None, None))],
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
    pub fn tick(&self) -> io::Result<()> {
        let mut r = self.shared.lock().map_err(|_| err("room unavailable"))?;
        if !r.packets.contains_key("rank") {
            return Ok(());
        }
        let Phase::Armed { round } = r.model.phase else {
            return Ok(());
        };
        if r.model.members.len() != 2 {
            return Ok(());
        }
        let now = Instant::now();
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
            "rank_all_dead"
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
            if slot == 0 { "rank" } else { "rank_Awin" }
        } else {
            return Ok(());
        };
        let mut next = r.model.clone();
        next.end(round)?;
        let mut effects = Vec::new();
        let returning = r.reply(
            "return",
            None,
            Some(room_wire::room_selector(
                &r.packets["return"].payload,
                180,
                r.selector,
            )?),
        );
        let slots = next
            .members
            .values()
            .map(|m| r.slot(m, next.host))
            .collect::<io::Result<Vec<_>>>()?;
        for peer in next.members.values() {
            for (delay, packets, complete) in [
                (
                    0,
                    vec![("rank175".into(), r.reply(alias, None, None))],
                    false,
                ),
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
        if let Some((source, round, digest)) = &batch.life_complete {
            let mut runtime = self.shared.lock().map_err(|_| err("room unavailable"))?;
            if runtime.model.current(*source)
                && runtime.model.phase == (Phase::Armed { round: *round })
            {
                runtime
                    .model
                    .members
                    .get_mut(&source.identity)
                    .expect("current source")
                    .life
                    .forward21_complete(digest);
            }
        }
        if let Some((recipient, round)) = batch.outcome_complete {
            self.shared
                .lock()
                .map_err(|_| err("room unavailable"))?
                .model
                .refreshed(recipient, round)?;
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
                    vec![("table238".into(), r.reply("table", None, None))],
                );
                batch.guard = Guard::Channel;
                effects.push((self.identity, batch));
            }
            81 | 82 => {
                let (title, selector) = if opcode == 81 {
                    (room_wire::create_title(body)?, room_wire::selector(body)?)
                } else {
                    room_wire::join_context(body)?;
                    (r.title.clone(), r.selector)
                };
                let key = next.enter(self.identity, self.connection, opcode == 81)?;
                let m = next.members[&self.identity].clone();
                let reply = if opcode == 81 {
                    r.reply(
                        "create",
                        None,
                        Some(room_wire::room_selector(
                            &r.packets["create"].payload,
                            86,
                            selector,
                        )?),
                    )
                } else {
                    r.reply(
                        "join",
                        None,
                        Some(room_wire::room_selector(
                            &room_wire::join(&r.packets["join"].payload, &m)?,
                            87,
                            selector,
                        )?),
                    )
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
                                            "listing",
                                            None,
                                            Some(room_wire::replace_listing_title(
                                                &r.packets["listing"].payload,
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
                if !room_wire::is_zero_setup(body)? {
                    return Err(err("unsupported setup585"));
                }
                let m = r.member(self.identity, self.connection)?;
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
                let mut peers = Vec::new();
                for peer in next.members.values() {
                    if peer.key != m.key {
                        if !r.rostered.contains(&(m.key, peer.key)) {
                            peers.push(("peer106".into(), r.player(peer, next.host)?));
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
                if !peers.is_empty() {
                    let peer = next
                        .members
                        .values()
                        .find(|peer| peer.key != m.key)
                        .expect("two members");
                    let mut batch = Runtime::member_batch(m.key, None, 3000, peers);
                    batch.subject = Some(peer.key);
                    effects.push((self.identity, batch));
                }
                let mut final_batch = Runtime::member_batch(
                    m.key,
                    None,
                    4000,
                    vec![
                        ("table238".into(), r.reply("table", None, None)),
                        ("map117".into(), r.reply("map", None, None)),
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
                            ("return91".into(), r.reply("listing", None, None)),
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
                let round = next.begin(m.key)?;
                next.pickups = pickups::Round::begin(round, &r.floor_catalog)?;
                for peer in next.members.values() {
                    effects.push((
                        peer.key.identity,
                        Runtime::member_batch(
                            peer.key,
                            Some(round),
                            0,
                            vec![
                                ("floor582".into(), r.reply("floor", None, None)),
                                ("start121".into(), r.reply("start", None, None)),
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
                        let mut endpoints = [(0, 0); 2];
                        for peer in next.members.values() {
                            let p = if peer.key == m.key {
                                port
                            } else {
                                *r.ports
                                    .get(&peer.key)
                                    .ok_or_else(|| err("missing current loading endpoint"))?
                            };
                            endpoints[usize::from(peer.slot)] = (r.ips[&peer.key.identity], p);
                        }
                        let key = crate::room::wire_key(round)?;
                        for peer in next.members.values() {
                            effects.push((
                                peer.key.identity,
                                Runtime::member_batch(
                                    peer.key,
                                    Some(round),
                                    0,
                                    vec![("go197".into(), r.reply("go", None, None))],
                                ),
                            ));
                            effects.push((
                                peer.key.identity,
                                Runtime::member_batch(
                                    peer.key,
                                    Some(round),
                                    1000,
                                    vec![("play196".into(), r.reply("play", None, None))],
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
                                            "peer",
                                            None,
                                            Some(room_wire::peer(
                                                &r.packets["peer"].payload,
                                                key,
                                                peer.slot,
                                                endpoints,
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
                if ![4, 16, 32].contains(&body.len()) {
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
                let member = r.member(self.identity, self.connection)?;
                let request = bombs::Request::parse(body)?;
                let result = if matches!(next.phase, Phase::Armed { .. })
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
                        let spawn = r.reply("start", Some(159), Some(request.spawn159(ticket.id)?));
                        let blast =
                            r.reply("start", Some(158), Some(bombs::explode158(ticket.id)?));
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
                let member = r.member(self.identity, self.connection)?;
                if body.len() != 7 {
                    return Err(err("known142 size"));
                }
                let slot = read_uint(&body[2..3], Width::U8).map_err(io::Error::other)?;
                let aux = read_uint(&body[3..], Width::U32).map_err(io::Error::other)?;
                terrain_notice = Some(if let Phase::Armed { round } = next.phase {
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
                    let reply = r.reply("start", Some(165), Some(request.confirmation()?));
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
                    if r.packets.contains_key("rank") {
                        let report = crate::life::Report::parse(body)?;
                        let now = Instant::now();
                        let last_blast = next.last_blast;
                        let decision = next
                            .members
                            .get_mut(&self.identity)
                            .expect("current member")
                            .life
                            .apply(m.slot, &report, now, last_blast);
                        if decision.forward {
                            let peer = next
                                .members
                                .values()
                                .find(|p| p.key != m.key)
                                .ok_or_else(|| err("missing life recipient"))?;
                            let mut batch = Runtime::member_batch(
                                peer.key,
                                Some(round),
                                0,
                                vec![(
                                    format!("life170-{}", report.subtype),
                                    r.reply("start", Some(170), report.forward.clone()),
                                )],
                            );
                            batch.guard = Guard::Life {
                                recipient: peer.key,
                                source: m.key,
                                round,
                                clear21: (report.subtype == 21)
                                    .then(|| (report.digest.clone(), now)),
                            };
                            if report.subtype == 21 {
                                batch.life_complete = Some((m.key, round, report.digest.clone()));
                            }
                            effects.push((peer.key.identity, batch));
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
            let listing = room_wire::replace_listing_title(&r.packets["listing"].payload, &title)?;
            r.packets
                .get_mut("listing")
                .expect("validated listing")
                .payload = listing;
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
mod tests {
    use super::*;
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
            model: Room::default(),
            packets: HashMap::new(),
            channels: HashMap::from([
                (
                    1,
                    Channel {
                        connection: 11,
                        sender: ta,
                    },
                ),
                (
                    2,
                    Channel {
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
        runtime.packets.insert("start".into(), reply.clone());
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
