//! Account-owned selection service. Synchronous lock order: hub -> room -> pool.
//! Network/DB awaits stay with the connection; watch notifications coalesce.
use crate::{
    account_bridge::RelayAuthority,
    room_io,
    room_registry::{Registry, RoomId, RoomKey},
    room_relay,
};
use kr_wire::kr00967 as wire;
use std::{
    collections::{BTreeSet, HashMap},
    io,
    net::Ipv4Addr,
    sync::{Arc, Mutex},
};
use tokio::sync::watch;

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub max_rooms: usize,
    #[serde(default)]
    pub round_profile: Option<room_io::ManagedRoundConfig>,
    #[serde(default)]
    pub player_choices: bool,
    #[serde(default)]
    pub capacity: Option<usize>,
    #[serde(default)]
    pub player_round: Option<room_io::PlayerRoundConfig>,
}
struct Owner {
    connection: u64,
    selected: Option<RoomKey>,
}
struct State {
    rooms: Registry<room_io::Shared>,
    owners: HashMap<u32, Owner>,
    next: u16,
}
pub(crate) struct Hub {
    state: Mutex<State>,
    prototype: room_io::Shared,
    max_rooms: usize,
    changes: watch::Sender<u64>,
    relay: Option<room_relay::Factory>,
    authenticated: Option<Arc<crate::authenticated_peer::Hub>>,
    epochs: Arc<crate::round_epochs::RoundEpochs>,
    round_profile: Option<Arc<room_io::ManagedRound>>,
    player_choices: bool,
    capacity: Option<usize>,
}
pub(crate) struct Channel {
    hub: Arc<Hub>,
    identity: u32,
    connection: u64,
    pub changes: watch::Receiver<u64>,
}
pub(crate) struct Catalog {
    pub tuples: BTreeSet<(u16, u32)>,
    pub packets: Vec<(u32, Vec<u8>)>,
}
pub(crate) enum Selection {
    Joined(room_io::Session),
    Full,
}
fn err(s: &str) -> io::Error {
    io::Error::other(s.to_owned())
}
impl Hub {
    pub(crate) fn new(
        cfg: &Config,
        prototype: room_io::Shared,
        relay: Option<room_relay::Factory>,
        authenticated: Option<Arc<crate::authenticated_peer::Hub>>,
    ) -> io::Result<Arc<Self>> {
        if !(2..=64).contains(&cfg.max_rooms) {
            return Err(err("managed room bound must be 2..64"));
        }
        if cfg.player_choices && cfg.round_profile.is_some() {
            return Err(err(
                "room choices require selection-only profile until Start/stat acceptance",
            ));
        }
        if cfg.player_round.is_some() && (cfg.round_profile.is_some() || !cfg.player_choices) {
            return Err(err(
                "player round requires choices and no legacy round profile",
            ));
        }
        room_io::validate_admission_capacity(
            &prototype,
            cfg.capacity,
            cfg.round_profile
                .as_ref()
                .map(|profile| profile.teams.len()),
        )?;
        let round_profile = cfg
            .round_profile
            .as_ref()
            .map(|cfg| room_io::load_managed_round(cfg, &prototype))
            .transpose()?;
        let round_profile = if let Some(players) = &cfg.player_round {
            Some(room_io::load_player_round(players, &prototype)?)
        } else {
            round_profile
        };
        let (changes, _) = watch::channel(0);
        let epochs = Arc::new(if authenticated.is_some() {
            crate::round_epochs::RoundEpochs::authenticated()
        } else {
            crate::round_epochs::RoundEpochs::default()
        });
        Ok(Arc::new(Self {
            state: Mutex::new(State {
                rooms: Registry::default(),
                owners: HashMap::new(),
                next: 1,
            }),
            prototype,
            max_rooms: cfg.max_rooms,
            changes,
            relay,
            authenticated,
            epochs,
            round_profile,
            player_choices: cfg.player_choices,
            capacity: cfg.capacity,
        }))
    }
    pub(crate) fn register(
        self: &Arc<Self>,
        identity: u32,
        connection: u64,
    ) -> io::Result<Channel> {
        let mut s = self
            .state
            .lock()
            .map_err(|_| err("room service unavailable"))?;
        if s.owners.contains_key(&identity) {
            return Err(err("duplicate account lobby channel"));
        }
        s.owners.insert(
            identity,
            Owner {
                connection,
                selected: None,
            },
        );
        Ok(Channel {
            hub: self.clone(),
            identity,
            connection,
            changes: self.changes.subscribe(),
        })
    }
    fn changed(&self) {
        self.changes.send_modify(|n| *n = n.wrapping_add(1));
    }
}
impl Channel {
    fn owner<'a>(&self, s: &'a State) -> io::Result<&'a Owner> {
        s.owners
            .get(&self.identity)
            .filter(|o| o.connection == self.connection)
            .ok_or_else(|| err("stale account lobby owner"))
    }
    pub(crate) fn select(
        &self,
        opcode: u32,
        body: &[u8],
        peer: Ipv4Addr,
        authority: RelayAuthority,
        character: Option<u16>,
    ) -> io::Result<Selection> {
        let mut s = self
            .hub
            .state
            .lock()
            .map_err(|_| err("room service unavailable"))?;
        if !authority.owns(self.identity, self.connection) {
            return Err(err("selection has no current account transport owner"));
        }
        if self.owner(&s)?.selected.is_some() {
            return Err(err("account already selected a room"));
        }
        let create = opcode == 81;
        let key = if create {
            wire::create81_title(body)?;
            wire::create81_selector(body)?;
            if s.rooms.entries().count() >= self.hub.max_rooms || s.next == 0 {
                return Err(err("room allocation exhausted"));
            }
            let session = s.next;
            s.next = session.checked_add(1).unwrap_or(0);
            s.rooms
                .insert_with(session, RoomId(u32::from(session)), |key| {
                    let shared = room_io::fork(
                        &self.hub.prototype,
                        key,
                        self.hub.epochs.clone(),
                        self.hub.round_profile.clone(),
                        self.hub.player_choices,
                        self.hub.capacity,
                    )?;
                    if let Some(factory) = &self.hub.relay {
                        room_io::install_relay(&shared, factory.owner(key))?;
                    }
                    if let Some(hub) = &self.hub.authenticated {
                        room_io::install_authenticated(
                            &shared,
                            crate::authenticated_room_relay::Owner::new(hub.clone(), key),
                        )?;
                    }
                    Ok(shared)
                })?
        } else if opcode == 82 {
            let choice = wire::join82_selection(body)?;
            s.rooms.selection(choice.session, RoomId(choice.room_id))?
        } else {
            return Err(err("invalid selection opcode"));
        };
        // Hub -> room lock order serializes this check with every selection.
        // No channel, membership, visit, tuple or round is allocated on failure.
        if !create && room_io::full_idle(s.rooms.get(key)?)? {
            return Ok(Selection::Full);
        }
        let attempt = (|| {
            let session = room_io::Session::open_with_authority(
                s.rooms.get(key)?,
                self.identity,
                self.connection,
                Some(peer),
                Some(authority),
            )?;
            session.packet(opcode, body)?;
            if let Some(character) = character {
                session.restore_character(character)?;
            }
            Ok(session)
        })();
        match attempt {
            Ok(session) => {
                s.owners
                    .get_mut(&self.identity)
                    .expect("current owner")
                    .selected = Some(key);
                self.hub.changed();
                Ok(Selection::Joined(session))
            }
            Err(error) => {
                if create {
                    let _ = s.rooms.remove_if(key, room_io::removable);
                }
                Err(error)
            }
        }
    }
    pub(crate) fn map_table(&self) -> io::Result<crate::Reply> {
        let state = self
            .hub
            .state
            .lock()
            .map_err(|_| err("room service unavailable"))?;
        if self.owner(&state)?.selected.is_some() {
            return Err(err("map cache request requires lobby owner"));
        }
        room_io::map_table_source(&self.hub.prototype)
    }
    pub(crate) fn catalog(&mut self, known: &BTreeSet<(u16, u32)>) -> io::Result<Catalog> {
        self.changes.borrow_and_update();
        let s = self
            .hub
            .state
            .lock()
            .map_err(|_| err("room service unavailable"))?;
        self.owner(&s)?;
        let tuples: BTreeSet<_> = s
            .rooms
            .entries()
            .map(|e| (e.key.session, e.key.id.0))
            .collect();
        let mut packets = Vec::new();
        for &(session, room_id) in known.difference(&tuples) {
            packets.push((
                90,
                wire::removal90_one(wire::JoinSelection { session, room_id })?,
            ));
        }
        packets.push((91, s.rooms.listing91()?));
        Ok(Catalog { tuples, packets })
    }
    /// Only after the native return241 batch completes. Caller drops its Session
    /// and clears pending work before permitting another selection.
    pub(crate) fn returned(&self) -> io::Result<()> {
        let mut s = self
            .hub
            .state
            .lock()
            .map_err(|_| err("room service unavailable"))?;
        let key = self.owner(&s)?.selected;
        if let Some(key) = key {
            let shared = s.rooms.get(key)?;
            room_io::detach(shared, self.identity, self.connection)?;
            s.owners
                .get_mut(&self.identity)
                .expect("current owner")
                .selected = None;
            if s.rooms.get(key).is_ok_and(room_io::removable) {
                s.rooms.remove(key)?;
            }
            self.hub.changed();
        }
        Ok(())
    }
}
impl Drop for Channel {
    fn drop(&mut self) {
        if self.returned().is_ok()
            && let Ok(mut s) = self.hub.state.lock()
            && self.owner(&s).is_ok()
        {
            s.owners.remove(&self.identity);
        }
    }
}
