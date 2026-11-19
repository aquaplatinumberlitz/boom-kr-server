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
    journal: Mutex<Option<crate::match_journal::Factory>>,
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
/// One native transport can own two independent accounts. Admission must
/// validate the complete roster before publishing any lobby owner.
pub(crate) struct LocalLobby {
    channels: Vec<Channel>,
    authorities: Vec<RelayAuthority>,
}
impl LocalLobby {
    pub(crate) fn select(&self, opcode: u32, body: &[u8], peer: Ipv4Addr) -> io::Result<Selection> {
        if !self.current() || self.channels.len() != 1 {
            return Err(err(
                "native room selection requires measured single-local layout",
            ));
        }
        self.channels[0].select(opcode, body, peer, self.authorities[0].clone(), None)
    }
    pub(crate) fn returned(&self) -> io::Result<()> {
        if !self.current() {
            return Err(err("local lobby owner retired"));
        }
        for channel in &self.channels {
            channel.returned()?;
        }
        Ok(())
    }
    pub(crate) fn map_table(&self) -> io::Result<crate::Reply> {
        if !self.current() {
            return Err(err("local lobby owner retired"));
        }
        self.channels[0].map_table()
    }
    pub(crate) fn catalog(&mut self) -> io::Result<Catalog> {
        self.catalog_since(&BTreeSet::new())
    }
    pub(crate) fn changed(&self) -> io::Result<bool> {
        self.channels[0]
            .changes
            .has_changed()
            .map_err(|_| err("local lobby catalog closed"))
    }
    pub(crate) fn catalog_since(&mut self, known: &BTreeSet<(u16, u32)>) -> io::Result<Catalog> {
        if !self.current() {
            return Err(err("local lobby owner retired"));
        }
        self.channels[0].catalog(known)
    }
    pub(crate) fn current(&self) -> bool {
        self.authorities.iter().all(RelayAuthority::current)
            && self.channels.iter().all(|channel| {
                channel
                    .hub
                    .state
                    .lock()
                    .is_ok_and(|state| channel.owner(&state).is_ok())
            })
    }
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
    pub(crate) fn register_local(
        self: &Arc<Self>,
        roster: &[crate::account_bridge::local::LocalAuthority],
    ) -> io::Result<LocalLobby> {
        if !(1..=2).contains(&roster.len()) {
            return Err(err("invalid local lobby roster"));
        }
        let connection = roster[0].connection();
        let authorities: Vec<_> = roster.iter().map(|owner| owner.relay_authority()).collect();
        let mut identities = std::collections::HashSet::new();
        if connection == 0
            || roster.iter().any(|owner| {
                owner.connection() != connection
                    || !owner.current()
                    || !identities.insert(owner.player().wire_id)
            })
        {
            return Err(err("local lobby roster has no common current transport"));
        }
        let mut state = self
            .state
            .lock()
            .map_err(|_| err("room service unavailable"))?;
        if identities
            .iter()
            .any(|identity| state.owners.contains_key(identity))
        {
            return Err(err("duplicate account lobby channel"));
        }
        // No await or fallible operation after the first insertion.
        let mut channels = Vec::with_capacity(roster.len());
        for owner in roster {
            let identity = owner.player().wire_id;
            state.owners.insert(
                identity,
                Owner {
                    connection,
                    selected: None,
                },
            );
            channels.push(Channel {
                hub: self.clone(),
                identity,
                connection,
                changes: self.changes.subscribe(),
            });
        }
        Ok(LocalLobby {
            channels,
            authorities,
        })
    }
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
            journal: Mutex::new(None),
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
    pub(crate) fn install_journal(&self, factory: crate::match_journal::Factory) -> io::Result<()> {
        let state = self.state.lock().map_err(|_| err("room hub unavailable"))?;
        let mut journal = self
            .journal
            .lock()
            .map_err(|_| err("journal factory unavailable"))?;
        if !state.owners.is_empty() || state.rooms.entries().next().is_some() || journal.is_some() {
            return Err(err("journal install requires pristine hub"));
        }
        *journal = Some(factory);
        Ok(())
    }
    pub(crate) fn cancel_journals(&self) -> io::Result<()> {
        let state = self.state.lock().map_err(|_| err("room hub unavailable"))?;
        for entry in state.rooms.entries() {
            room_io::cancel_journal(&entry.value)?;
        }
        Ok(())
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
                    if let Some(factory) = &*self
                        .hub
                        .journal
                        .lock()
                        .map_err(|_| err("journal factory unavailable"))?
                    {
                        room_io::install_journal(&shared, factory.owner())?;
                    }
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

#[cfg(test)]
mod local_tests {
    use super::*;
    use crate::account_bridge::tests::{Temp, admin, now, rt, worker};
    use crate::account_bridge::{
        local::LocalSession,
        local::tests::{channel, login},
    };
    use crate::accounts::TEST_SERIAL;
    #[test]
    fn local_lobby_admission_is_atomic_and_retired_handles_cannot_reclaim_reused_connections() {
        let _serial = TEST_SERIAL.lock().unwrap();
        let (temp, a, b) = Temp::new();
        rt().block_on(async {
            let worker = worker(&temp).await;
            let hub = Hub::new(
                &Config {
                    max_rooms: 8,
                    round_profile: None,
                    player_choices: false,
                    capacity: None,
                    player_round: None,
                },
                crate::room_io::registry_foundation_tests::empty(0x77),
                None,
                None,
            )
            .unwrap();
            let mut owner = LocalSession::new(worker.client(), 11).unwrap();
            let rows = owner
                .login(&login(&["Alice", "Bob"]), 60, None)
                .await
                .unwrap();
            owner.redirected(11).unwrap();
            let handles = owner.channel(&channel(&rows), 22).await.unwrap();
            let conflict = hub.register(b.wire_id, 99).unwrap();
            assert!(hub.register_local(&handles).is_err());
            assert!(!hub.state.lock().unwrap().owners.contains_key(&a.wire_id));
            assert_eq!(hub.state.lock().unwrap().owners.len(), 1);
            drop(conflict);
            let lobby = hub.register_local(&handles).unwrap();
            assert!(lobby.current());
            assert_eq!(hub.state.lock().unwrap().owners.len(), 2);
            assert!(lobby.select(81, &[], Ipv4Addr::LOCALHOST).is_err());
            assert!(
                hub.state
                    .lock()
                    .unwrap()
                    .owners
                    .values()
                    .all(|owner| owner.selected.is_none())
            );
            assert_eq!(hub.state.lock().unwrap().rooms.entries().count(), 0);
            let generation = handles[0].relay_authority().generation();
            temp.store().revoke(&admin(), b.account_id, now()).unwrap();
            assert!(handles[0].inspect().await.is_err());
            assert!(!lobby.current());
            owner.finish().await.unwrap();
            drop(lobby);
            assert!(hub.state.lock().unwrap().owners.is_empty());
            let mut next = LocalSession::new(worker.client(), 11).unwrap();
            let rows = next
                .login(&login(&["Alice", "Bob"]), 60, None)
                .await
                .unwrap();
            next.redirected(11).unwrap();
            let fresh = next.channel(&channel(&rows), 22).await.unwrap();
            assert_ne!(generation, fresh[0].relay_authority().generation());
            assert!(hub.register_local(&handles).is_err());
            assert!(hub.state.lock().unwrap().owners.is_empty());
            let fresh_lobby = hub.register_local(&fresh).unwrap();
            assert!(fresh_lobby.current());
            assert!(handles.iter().all(|old| !old.relay_authority().current()));
            drop(fresh_lobby);
            next.finish().await.unwrap();
            worker.shutdown().await.unwrap();
        });
    }
}
