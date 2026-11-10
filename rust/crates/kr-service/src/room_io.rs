//! Fixed-room adapters with bounded recipient queues and membership fences.
use super::{ActorProfile, LobbyPacketPin, Reply};
use crate::{
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
}
pub type Shared = Arc<Mutex<Runtime>>;
struct Channel {
    connection: u64,
    sender: mpsc::Sender<Batch>,
}
#[derive(Clone)]
pub enum Guard {
    Channel,
    Member(Membership, Option<u64>),
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
    Ok(Arc::new(Mutex::new(Runtime {
        model: Room::default(),
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
            if next.leave(m.key).is_ok() {
                let effects = self.departure(m.key, &next);
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
    pub fn handles(opcode: u32) -> bool {
        matches!(
            opcode,
            81 | 82 | 585 | 86 | 95 | 104 | 99 | 528 | 127 | 96 | 266 | 190
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
            Guard::Member(key, round) => r.model.deliverable(key, round),
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
        if let Some(key) = batch.setup {
            self.shared
                .lock()
                .map_err(|_| err("room unavailable"))?
                .model
                .setup(key)?;
        }
        Ok(())
    }
    pub fn packet(&self, opcode: u32, body: &[u8]) -> io::Result<bool> {
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
                    return Err(err("duplicate setup585"));
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
                        let key = 12345u32
                            .checked_add(
                                u32::try_from(round).map_err(|_| err("round key exhausted"))?,
                            )
                            .ok_or_else(|| err("round key exhausted"))?;
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
            127 => {
                let m = r.member(self.identity, self.connection)?;
                if !(1..=4096).contains(&body.len()) {
                    return Err(err("M2 report observation size"));
                }
                if next.phase == Phase::Idle {
                    next.observe_after_abort(m.key)?;
                    dropped_after_abort = true;
                } else if !matches!(next.phase, Phase::Armed { .. }) {
                    return Err(err("M2 report observation phase"));
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
        Ok(dropped_after_abort)
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
        let (ta, mut ra) = mpsc::channel(1);
        let (tb, mut rb) = mpsc::channel(1);
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
        };
        let mut next = runtime.model.clone();
        let key = next.enter(1, 11, true).unwrap();
        let batch = || Runtime::member_batch(key, None, 0, vec![("test".into(), reply.clone())]);
        tb.try_send(batch())
            .unwrap_or_else(|_| panic!("fill bounded queue"));
        assert!(
            runtime
                .publish(next.clone(), vec![(1, batch()), (2, batch())])
                .is_err()
        );
        assert!(runtime.model.members.is_empty());
        assert!(ra.try_recv().is_err());
        assert!(rb.try_recv().is_ok());
        assert!(
            runtime
                .publish(next, vec![(1, batch()), (2, batch())])
                .is_ok()
        );
        assert!(runtime.model.current(key));
        assert!(ra.try_recv().is_ok());
        assert!(rb.try_recv().is_ok());
    }
}
