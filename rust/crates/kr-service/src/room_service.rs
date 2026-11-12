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
fn err(s: &str) -> io::Error {
    io::Error::other(s.to_owned())
}
impl Hub {
    pub(crate) fn new(
        cfg: &Config,
        prototype: room_io::Shared,
        relay: Option<room_relay::Factory>,
    ) -> io::Result<Arc<Self>> {
        if !(2..=64).contains(&cfg.max_rooms) {
            return Err(err("managed room bound must be 2..64"));
        }
        let (changes, _) = watch::channel(0);
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
    ) -> io::Result<room_io::Session> {
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
                    let shared = room_io::fork(&self.hub.prototype, key)?;
                    if let Some(factory) = &self.hub.relay {
                        room_io::install_relay(&shared, factory.owner(key))?;
                    }
                    Ok(shared)
                })?
        } else if opcode == 82 {
            let choice = wire::join82_selection(body)?;
            s.rooms.selection(choice.session, RoomId(choice.room_id))?
        } else {
            return Err(err("invalid selection opcode"));
        };
        let attempt = (|| {
            let session = room_io::Session::open_with_authority(
                s.rooms.get(key)?,
                self.identity,
                self.connection,
                Some(peer),
                Some(authority),
            )?;
            session.packet(opcode, body)?;
            Ok(session)
        })();
        match attempt {
            Ok(session) => {
                s.owners
                    .get_mut(&self.identity)
                    .expect("current owner")
                    .selected = Some(key);
                self.hub.changed();
                Ok(session)
            }
            Err(error) => {
                if create {
                    let _ = s.rooms.remove_if(key, room_io::removable);
                }
                Err(error)
            }
        }
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
