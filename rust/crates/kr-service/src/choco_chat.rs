//! Local F5-6 routing policy. Channel names are logical; no CHOCO wire format
//! or native client behavior is assumed. Run this synchronous scaffold with
//! its Schema 9 Store on a single bounded owner, off the async I/O loop.
//! The opt-in F5 checks are local design policy (f5-extended-social-matrix),
//! not KR 00967 packet handlers or persistence for the extended features.

use crate::{
    accounts::{
        self, CHAT_MAX_BYTES, CHAT_WINDOW_CAP, CHAT_WINDOW_SECONDS, ChatPermission, GuildMember,
        Store, Token,
    },
    room_registry::RoomId,
};
use kr_wire::kr00967::text949x;
use serde::{Deserialize, Serialize};
use std::{
    borrow::{Borrow, Cow},
    collections::{BTreeMap, VecDeque},
    fs, io,
    num::{NonZeroU16, NonZeroU32},
    path::{Component, Path},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

/// Bounds online registrations plus disconnected players with recent sends.
pub const CHAT_PLAYER_CAP: usize = 256;
static NEXT_SESSION: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChocoChatChannel {
    All,
    Guild,
    /// A player ID already resolved by the service.
    Whisper(i64),
    Team,
    Shout,
}

/// Presence comes from service state and must be refreshed on room, team, or
/// guild changes. The Store roster also has to agree with guild presence.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ChocoChatPresence {
    pub room_id: Option<RoomId>,
    pub team_id: Option<u8>,
    pub guild_id: Option<i64>,
}

/// An opaque in-process registration handle. Bind it to its authenticated
/// connection; it is never a field accepted from a chat request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChocoChatSession {
    player_id: i64,
    generation: u64,
}

impl ChocoChatSession {
    pub fn player_id(self) -> i64 {
        self.player_id
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChocoChatError {
    Unauthenticated,
    NotRegistered,
    AlreadyRegistered,
    Capacity,
    InvalidPresence,
    InvalidLength,
    InvalidText,
    Muted,
    Blocked,
    RateLimited,
    TargetUnavailable,
    NoGuild,
    NoTeam,
    Store(accounts::Error),
}

use ChocoChatError as Error;
type Result<T> = std::result::Result<T, Error>;

/// Opt-in local policy. `Default` disables all extended paths; the legacy
/// `route` API remains available under its existing Schema 9 policy.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct F5SocialPolicy {
    pub whisper: bool,
    pub guild_chat: bool,
    pub megaphone: Option<F5MegaphonePolicy>,
    pub note_inbox_cap: Option<NonZeroU16>,
    pub couple: bool,
    pub plaza_zones: BTreeMap<F5PlazaZone, F5PlazaLimits>,
}

const F5_MAX_QUOTA: usize = 64;
const F5_MAX_WINDOW_SECONDS: i64 = 86_400;
const F5_MAX_INBOX_CAP: u16 = 100;
const F5_MAX_SEATS: u16 = 256;
pub const F5_ABUSE_EVENT_CAP: usize = 256;
const MAX_F5_POLICY_BYTES: u64 = 4096;

impl F5SocialPolicy {
    /// Load a bounded, strictly typed policy from a trusted local JSON file.
    /// Every path component is checked so symlinks fail closed.
    pub fn load_from_path(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref();
        if path.as_os_str().is_empty()
            || path
                .components()
                .any(|part| matches!(part, Component::ParentDir))
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid f5_social_policy path",
            ));
        }
        for parent in path
            .ancestors()
            .skip(1)
            .filter(|p| !p.as_os_str().is_empty())
        {
            if !fs::symlink_metadata(parent)?.is_dir() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "f5_social_policy parents must be directories without symlinks",
                ));
            }
        }
        if !fs::symlink_metadata(path)?.is_file() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "f5_social_policy must be a regular file without symlinks",
            ));
        }
        if fs::metadata(path)?.len() > MAX_F5_POLICY_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "f5_social_policy file size out of bounds",
            ));
        }
        let bytes = fs::read(path)?;
        let policy: Self = serde_json::from_slice(&bytes)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        policy.validate_bounds()?;
        Ok(policy)
    }

    fn validate_bounds(&self) -> io::Result<()> {
        if self.megaphone.is_some_and(|policy| {
            !(policy.global || policy.channel)
                || !(1..=F5_MAX_WINDOW_SECONDS).contains(&policy.window_seconds)
                || !(1..=policy.window_seconds).contains(&policy.cooldown_seconds)
                || !(1..=F5_MAX_QUOTA).contains(&policy.quota)
        }) || self
            .note_inbox_cap
            .is_some_and(|cap| cap.get() > F5_MAX_INBOX_CAP)
            || self.plaza_zones.values().any(|zone| {
                usize::from(zone.max_occupants.get()) > CHAT_PLAYER_CAP || zone.seats > F5_MAX_SEATS
            })
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "f5_social_policy limits are out of bounds",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct F5MegaphonePolicy {
    pub global: bool,
    pub channel: bool,
    /// Both scopes share the same cooldown and sliding quota per account.
    pub cooldown_seconds: i64,
    pub window_seconds: i64,
    pub quota: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum F5MegaphoneScope {
    Global,
    /// The sender's current service channel; never a channel ID from the request.
    Channel,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum F5ChatRoute {
    Whisper(i64),
    Guild,
    Megaphone(F5MegaphoneScope),
}

/// Local text input. 949x uses the patched-client codec, not CHOCO framing.
#[derive(Clone, Copy)]
pub enum F5Message<'a> {
    Utf8(&'a str),
    Text949x(&'a [u8]),
}

impl F5Message<'_> {
    fn submitted_bytes(self) -> usize {
        match self {
            Self::Utf8(text) => text.len(),
            Self::Text949x(bytes) => bytes.len(),
        }
    }

    /// Check the input length before decoding/allocating, then check both
    /// representations. Return only the byte count charged to Schema 9.
    fn validate(self) -> Result<usize> {
        validate_chat_length(self.submitted_bytes())?;
        let text = match self {
            Self::Utf8(text) => Cow::Borrowed(text),
            Self::Text949x(bytes) => {
                Cow::Owned(text949x::decode(bytes).map_err(|_| Error::InvalidText)?)
            }
        };
        validate_chat_length(text.len())?;
        if !valid_chat_text(&text) {
            return Err(Error::InvalidText);
        }
        let encoded = text949x::encode(&text).map_err(|_| Error::InvalidText)?;
        validate_chat_length(encoded.len())?;
        if let Self::Text949x(bytes) = self
            && bytes != encoded.as_slice()
        {
            return Err(Error::InvalidText);
        }
        Ok(encoded.len())
    }
}

fn validate_chat_length(bytes: usize) -> Result<()> {
    if (1..=CHAT_MAX_BYTES).contains(&bytes) {
        Ok(())
    } else {
        Err(Error::InvalidLength)
    }
}

fn valid_chat_text(message: &str) -> bool {
    !message.trim().is_empty() && !message.chars().any(char::is_control)
}

/// Authoritative recipient and current count, to be checked in the same
/// transaction as storage. The recipient need not have a registered session.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct F5NoteInbox {
    pub recipient_id: i64,
    pub messages: usize,
    pub unread: usize,
}

impl F5NoteInbox {
    /// Create an empty inbox. A missing cap keeps this local runtime disabled.
    pub fn new<P: Borrow<F5SocialPolicy>>(policy: P, recipient_id: i64) -> F5Result<Self> {
        let policy = policy.borrow();
        if policy.note_inbox_cap.is_none() {
            return Err(F5Error::Disabled);
        }
        if recipient_id <= 0 {
            return Err(F5Error::Chat(Error::TargetUnavailable));
        }
        Ok(Self {
            recipient_id,
            messages: 0,
            unread: 0,
        })
    }

    /// Add one note after validating both identities and the bounded text.
    pub fn enqueue(
        &mut self,
        policy: &F5SocialPolicy,
        sender_id: i64,
        message: F5Message<'_>,
    ) -> F5Result<usize> {
        let cap = policy.note_inbox_cap.ok_or(F5Error::Disabled)?;
        if self.recipient_id <= 0 || sender_id <= 0 || sender_id == self.recipient_id {
            return Err(F5Error::Chat(Error::TargetUnavailable));
        }
        if cap.get() > F5_MAX_INBOX_CAP || self.messages > usize::from(cap.get()) {
            return Err(F5Error::InvalidPolicy);
        }
        let bytes = message.validate()?;
        if self.messages >= usize::from(cap.get()) {
            return Err(F5Error::InboxFull);
        }
        self.messages = self.messages.saturating_add(1);
        self.unread = self.unread.saturating_add(1);
        Ok(bytes)
    }

    pub fn push(
        &mut self,
        policy: &F5SocialPolicy,
        sender_id: i64,
        message: &str,
    ) -> F5Result<usize> {
        self.enqueue(policy, sender_id, F5Message::Utf8(message))
    }

    pub fn add_note(
        &mut self,
        policy: &F5SocialPolicy,
        sender_id: i64,
        message: &str,
    ) -> F5Result<usize> {
        self.push(policy, sender_id, message)
    }

    pub fn mark_read(&mut self, count: usize) -> F5Result<()> {
        if count > self.unread {
            return Err(F5Error::InvalidTransition);
        }
        self.unread -= count;
        Ok(())
    }

    pub fn unread_count(&self) -> usize {
        self.unread
    }

    pub fn len(&self) -> usize {
        self.messages
    }

    pub fn is_empty(&self) -> bool {
        self.messages == 0
    }

    pub fn mark_all_read(&mut self) {
        self.unread = 0;
    }

    pub fn delete(&mut self, count: usize) -> F5Result<()> {
        if count > self.messages {
            return Err(F5Error::InvalidTransition);
        }
        self.messages -= count;
        self.unread = self.unread.min(self.messages);
        Ok(())
    }

    pub fn delete_all(&mut self) {
        self.messages = 0;
        self.unread = 0;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct F5CoupleState {
    partner: i64,
    ring: bool,
}

/// Bounded process-local couple state. It never stores names or other private data.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct F5CoupleRuntime {
    enabled: bool,
    pairs: BTreeMap<i64, F5CoupleState>,
}

impl F5CoupleRuntime {
    pub fn new<P: Borrow<F5SocialPolicy>>(policy: P) -> Self {
        let policy = policy.borrow();
        Self {
            enabled: policy.couple,
            pairs: BTreeMap::new(),
        }
    }

    fn gate(&self) -> F5Result<()> {
        if self.enabled {
            Ok(())
        } else {
            Err(F5Error::Disabled)
        }
    }

    pub fn pair(&mut self, sender_id: i64, recipient_id: i64) -> F5Result<()> {
        self.gate()?;
        if sender_id <= 0 || recipient_id <= 0 || sender_id == recipient_id {
            return Err(F5Error::Chat(Error::TargetUnavailable));
        }
        if self.pairs.contains_key(&sender_id) || self.pairs.contains_key(&recipient_id) {
            return Err(F5Error::PartnerOccupied);
        }
        self.pairs.insert(
            sender_id,
            F5CoupleState {
                partner: recipient_id,
                ring: false,
            },
        );
        self.pairs.insert(
            recipient_id,
            F5CoupleState {
                partner: sender_id,
                ring: false,
            },
        );
        Ok(())
    }

    pub fn equip_ring(&mut self, player_id: i64) -> F5Result<()> {
        self.gate()?;
        let partner = self
            .pairs
            .get(&player_id)
            .ok_or(F5Error::NotPaired)?
            .partner;
        if let Some(state) = self.pairs.get_mut(&player_id) {
            state.ring = true;
        }
        if let Some(state) = self.pairs.get_mut(&partner) {
            state.ring = true;
        }
        Ok(())
    }

    pub fn divorce(&mut self, player_id: i64) -> F5Result<()> {
        self.gate()?;
        let partner = self
            .pairs
            .get(&player_id)
            .ok_or(F5Error::NotPaired)?
            .partner;
        self.pairs.remove(&player_id);
        self.pairs.remove(&partner);
        Ok(())
    }

    pub fn ring(&mut self, player_id: i64) -> F5Result<()> {
        self.equip_ring(player_id)
    }

    pub fn unpair(&mut self, player_id: i64) -> F5Result<()> {
        self.divorce(player_id)
    }

    pub fn partner(&self, player_id: i64) -> Option<i64> {
        self.pairs.get(&player_id).map(|state| state.partner)
    }

    pub fn has_ring(&self, player_id: i64) -> bool {
        self.pairs.get(&player_id).is_some_and(|state| state.ring)
    }

    pub fn is_paired(&self, player_id: i64) -> bool {
        self.pairs.contains_key(&player_id)
    }

    pub fn couple_count(&self) -> usize {
        self.pairs.len() / 2
    }

    pub fn active(&self) -> bool {
        self.enabled && !self.pairs.is_empty()
    }

    pub fn active_couples(&self) -> usize {
        self.couple_count()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct F5PlazaState {
    occupants: BTreeMap<i64, ()>,
    seats: BTreeMap<u16, i64>,
}

/// Bounded process-local plaza presence and cafe seating.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct F5PlazaRuntime {
    zones: BTreeMap<F5PlazaZone, (F5PlazaLimits, F5PlazaState)>,
}

impl F5PlazaRuntime {
    pub fn new<P: Borrow<F5SocialPolicy>>(policy: P) -> Self {
        let policy = policy.borrow();
        let zones = policy
            .plaza_zones
            .iter()
            .filter(|(_, limits)| {
                usize::from(limits.max_occupants.get()) <= CHAT_PLAYER_CAP
                    && limits.seats <= F5_MAX_SEATS
            })
            .map(|(&zone, &limits)| (zone, (limits, F5PlazaState::default())))
            .collect();
        Self { zones }
    }

    fn zone_mut(&mut self, zone: F5PlazaZone) -> F5Result<&mut (F5PlazaLimits, F5PlazaState)> {
        self.zones.get_mut(&zone).ok_or(F5Error::Disabled)
    }

    fn zone(&self, zone: F5PlazaZone) -> F5Result<&(F5PlazaLimits, F5PlazaState)> {
        self.zones.get(&zone).ok_or(F5Error::Disabled)
    }

    pub fn enter(&mut self, zone: F5PlazaZone, player_id: i64) -> F5Result<()> {
        if player_id <= 0 {
            return Err(F5Error::Chat(Error::TargetUnavailable));
        }
        let (limits, state) = self.zone_mut(zone)?;
        if state.occupants.contains_key(&player_id) {
            return Err(F5Error::InvalidTransition);
        }
        if state.occupants.len() >= usize::from(limits.max_occupants.get()) {
            return Err(F5Error::PlazaFull);
        }
        state.occupants.insert(player_id, ());
        Ok(())
    }

    pub fn leave(&mut self, zone: F5PlazaZone, player_id: i64) -> F5Result<()> {
        let (_, state) = self.zone_mut(zone)?;
        if state.occupants.remove(&player_id).is_none() {
            return Err(F5Error::NotInPlaza);
        }
        state.seats.retain(|_, occupant| *occupant != player_id);
        Ok(())
    }

    pub fn sit(&mut self, zone: F5PlazaZone, player_id: i64, seat: u16) -> F5Result<()> {
        let (limits, state) = self.zone_mut(zone)?;
        if !state.occupants.contains_key(&player_id) {
            return Err(F5Error::NotInPlaza);
        }
        if seat >= limits.seats {
            return Err(F5Error::InvalidSeat);
        }
        if state.seats.values().any(|occupant| *occupant == player_id) {
            return Err(F5Error::InvalidTransition);
        }
        if state.seats.contains_key(&seat) {
            return Err(F5Error::SeatUnavailable);
        }
        state.seats.insert(seat, player_id);
        Ok(())
    }

    pub fn stand(&mut self, zone: F5PlazaZone, player_id: i64) -> F5Result<()> {
        let (_, state) = self.zone_mut(zone)?;
        state.seats.retain(|_, occupant| *occupant != player_id);
        Ok(())
    }

    pub fn occupants(&self, zone: F5PlazaZone) -> F5Result<usize> {
        Ok(self.zone(zone)?.1.occupants.len())
    }

    pub fn join(&mut self, zone: F5PlazaZone, player_id: i64) -> F5Result<()> {
        self.enter(zone, player_id)
    }

    pub fn exit(&mut self, zone: F5PlazaZone, player_id: i64) -> F5Result<()> {
        self.leave(zone, player_id)
    }

    pub fn occupant_count(&self, zone: F5PlazaZone) -> F5Result<usize> {
        self.occupants(zone)
    }

    pub fn seat(&mut self, zone: F5PlazaZone, player_id: i64, seat: u16) -> F5Result<()> {
        self.sit(zone, player_id, seat)
    }

    pub fn unseat(&mut self, zone: F5PlazaZone, player_id: i64) -> F5Result<()> {
        self.stand(zone, player_id)
    }

    pub fn seated(&self, zone: F5PlazaZone) -> F5Result<usize> {
        Ok(self.zone(zone)?.1.seats.len())
    }

    pub fn active_zones(&self) -> usize {
        self.zones
            .values()
            .filter(|(_, state)| !state.occupants.is_empty())
            .count()
    }

    pub fn total_occupants(&self) -> usize {
        self.zones
            .values()
            .map(|(_, state)| state.occupants.len())
            .sum()
    }

    pub fn total_seated(&self) -> usize {
        self.zones
            .values()
            .map(|(_, state)| state.seats.len())
            .sum()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct F5SocialRuntimeSnapshot {
    pub note_inboxes: usize,
    pub notes: usize,
    pub unread_notes: usize,
    pub couples: usize,
    pub ringed_couples: usize,
    pub plaza_occupants: usize,
    pub plaza_seated: usize,
    pub active_plaza_zones: usize,
    pub notes_enabled: bool,
    pub couple_enabled: bool,
    pub plaza_enabled: bool,
}

impl F5SocialRuntimeSnapshot {
    pub fn new(
        policy: &F5SocialPolicy,
        inboxes: &[F5NoteInbox],
        couples: &F5CoupleRuntime,
        plaza: &F5PlazaRuntime,
    ) -> Self {
        let ringed_couples = couples.pairs.values().filter(|state| state.ring).count() / 2;
        Self {
            note_inboxes: inboxes.len(),
            notes: inboxes.iter().map(|inbox| inbox.messages).sum(),
            unread_notes: inboxes.iter().map(|inbox| inbox.unread).sum(),
            couples: couples.couple_count(),
            ringed_couples,
            plaza_occupants: plaza.total_occupants(),
            plaza_seated: plaza.total_seated(),
            active_plaza_zones: plaza.active_zones(),
            notes_enabled: policy.note_inbox_cap.is_some(),
            couple_enabled: policy.couple,
            plaza_enabled: !plaza.zones.is_empty(),
        }
    }

    pub fn from_parts(
        policy: &F5SocialPolicy,
        inboxes: &[F5NoteInbox],
        couples: &F5CoupleRuntime,
        plaza: &F5PlazaRuntime,
    ) -> Self {
        Self::new(policy, inboxes, couples, plaza)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum F5CoupleAction {
    /// Read both partner records under the same write transaction used to
    /// create a pairing. `None` means a confirmed absence, not an unread row.
    Pair {
        sender_id: i64,
        recipient_id: i64,
        sender_partner: Option<i64>,
        recipient_partner: Option<i64>,
    },
    EquipRing,
    AwardExp,
    AwardLucci,
}

/// Pack family names from the matrix. These are not numeric wire IDs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum F5PlazaZone {
    LobbySquare,
    LobbyCaffe,
    LobbyClothing,
    LobbyJewelry,
    LobbyPet,
    Square,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct F5PlazaLimits {
    pub max_occupants: NonZeroU16,
    /// Seats are zero based. Zero disables seating in this family.
    pub seats: u16,
}

/// Counts, presence, and seat occupancy come from the owner of the destination
/// map instance. The owner must validate and apply against the same state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum F5PlazaAction {
    Enter {
        occupants: usize,
    },
    Sit {
        present: bool,
        seat: u16,
        taken: bool,
        already_seated: bool,
    },
    ClaimCafeReward,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum F5UnmeasuredRule {
    CoupleRing,
    CoupleExp,
    CoupleLucci,
    CafeReward,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum F5SocialError {
    Disabled,
    InvalidPolicy,
    NoServiceChannel,
    MegaphoneCooldown,
    MegaphoneQuota,
    InboxFull,
    PartnerOccupied,
    PlazaFull,
    NotInPlaza,
    InvalidSeat,
    SeatUnavailable,
    InvalidTransition,
    NotPaired,
    UnmeasuredKrOpcode(u16),
    UnmeasuredRule(F5UnmeasuredRule),
    Chat(ChocoChatError),
}

type F5Result<T> = std::result::Result<T, F5SocialError>;

impl From<ChocoChatError> for F5SocialError {
    fn from(error: ChocoChatError) -> Self {
        Self::Chat(error)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum F5MessageKind {
    Whisper,
    Guild,
    Megaphone(F5MegaphoneScope),
    OfflineNote,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum F5AbuseReason {
    InvalidLength,
    InvalidText,
    Muted,
    Blocked,
    RateLimited,
    MegaphoneCooldown,
    MegaphoneQuota,
}

impl F5AbuseReason {
    fn for_error(error: F5SocialError) -> Option<Self> {
        match error {
            F5Error::Chat(Error::InvalidLength) => Some(Self::InvalidLength),
            F5Error::Chat(Error::InvalidText) => Some(Self::InvalidText),
            F5Error::Chat(Error::Muted) => Some(Self::Muted),
            F5Error::Chat(Error::Blocked) => Some(Self::Blocked),
            F5Error::Chat(Error::RateLimited) => Some(Self::RateLimited),
            F5Error::MegaphoneCooldown => Some(Self::MegaphoneCooldown),
            F5Error::MegaphoneQuota => Some(Self::MegaphoneQuota),
            _ => None,
        }
    }
}

/// Bounded process-local audit; Schema 9 also records its own chat denials.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct F5AbuseEvent {
    pub sender_id: i64,
    pub recipient_id: Option<i64>,
    pub kind: F5MessageKind,
    pub reason: F5AbuseReason,
    pub submitted_bytes: usize,
    pub accepted_in_window: usize,
    pub at_utc: i64,
}

impl From<accounts::Error> for Error {
    fn from(error: accounts::Error) -> Self {
        match error {
            accounts::Error::InvalidSession | accounts::Error::InvalidCredentials => {
                Self::Unauthenticated
            }
            error => Self::Store(error),
        }
    }
}

impl ChocoChatPresence {
    fn validate(self) -> Result<()> {
        if self.room_id == Some(RoomId(0))
            || (self.team_id.is_some() && self.room_id.is_none())
            || self.guild_id.is_some_and(|id| id <= 0)
        {
            return Err(Error::InvalidPresence);
        }
        Ok(())
    }
}

struct Online {
    handle: ChocoChatSession,
    token: Arc<Token>,
    presence: ChocoChatPresence,
    service_channel: Option<NonZeroU32>,
}

#[derive(Default)]
struct PlayerState {
    online: Option<Online>,
    broadcasts: Vec<i64>,
    f5_sends: SlidingWindow,
    megaphone_sends: SlidingWindow,
}

#[derive(Default)]
pub struct ChocoChatRouter {
    players: BTreeMap<i64, PlayerState>,
    f5_policy: F5SocialPolicy,
    f5_abuse_events: VecDeque<F5AbuseEvent>,
}

#[derive(Default)]
struct SlidingWindow(VecDeque<i64>);

impl SlidingWindow {
    fn prune(&mut self, now_utc: i64, seconds: i64) {
        let cutoff = now_utc - seconds;
        while self.0.front().is_some_and(|&at| at <= cutoff) {
            self.0.pop_front();
        }
    }
}

impl PlayerState {
    fn has_recent_sends(&self) -> bool {
        !(self.broadcasts.is_empty()
            && self.f5_sends.0.is_empty()
            && self.megaphone_sends.0.is_empty())
    }
}

enum Audience {
    Everyone,
    Team(RoomId, u8),
    Guild(i64, Vec<GuildMember>),
    ServiceChannel(NonZeroU32),
}

enum F5Delivery {
    Direct(ChocoChatSession),
    Broadcast(Audience),
    Note(F5NoteInbox),
}

#[derive(Clone, Copy)]
enum F5Destination {
    Chat(F5ChatRoute),
    Note(F5NoteInbox),
}

impl F5Destination {
    fn metadata(self) -> (F5MessageKind, Option<i64>) {
        match self {
            Self::Chat(F5ChatRoute::Whisper(id)) => (F5MessageKind::Whisper, Some(id)),
            Self::Chat(F5ChatRoute::Guild) => (F5MessageKind::Guild, None),
            Self::Chat(F5ChatRoute::Megaphone(scope)) => (F5MessageKind::Megaphone(scope), None),
            Self::Note(inbox) => (F5MessageKind::OfflineNote, Some(inbox.recipient_id)),
        }
    }
}

use F5SocialError as F5Error;

impl ChocoChatRouter {
    /// Use one router per service owner; replacing it discards process-local
    /// sliding windows and audit. Policy stays fixed over this router's lifetime.
    pub fn with_f5_policy(policy: F5SocialPolicy) -> F5Result<Self> {
        if policy.megaphone.is_some_and(|p| {
            !(p.global || p.channel)
                || !(1..=F5_MAX_WINDOW_SECONDS).contains(&p.window_seconds)
                || !(1..=p.window_seconds).contains(&p.cooldown_seconds)
                || !(1..=F5_MAX_QUOTA).contains(&p.quota)
        }) || policy
            .note_inbox_cap
            .is_some_and(|cap| cap.get() > F5_MAX_INBOX_CAP)
            || policy.plaza_zones.values().any(|zone| {
                usize::from(zone.max_occupants.get()) > CHAT_PLAYER_CAP || zone.seats > F5_MAX_SEATS
            })
        {
            return Err(F5Error::InvalidPolicy);
        }
        Ok(Self {
            f5_policy: policy,
            ..Self::default()
        })
    }

    fn prune_players(&mut self, now_utc: i64) {
        let cutoff = now_utc - CHAT_WINDOW_SECONDS;
        let megaphone = self.f5_policy.megaphone;
        self.players.retain(|_, state| {
            state.broadcasts.retain(|&at| at > cutoff);
            state.f5_sends.prune(now_utc, CHAT_WINDOW_SECONDS);
            if let Some(policy) = megaphone {
                state.megaphone_sends.prune(now_utc, policy.window_seconds);
            }
            state.online.is_some() || state.has_recent_sends()
        });
    }

    /// Register one live connection per player. All UTC timestamps come from
    /// the service clock used by the Store. The owner unregisters on close.
    pub fn register_session(
        &mut self,
        store: &mut Store,
        player_id: i64,
        token: Arc<Token>,
        presence: ChocoChatPresence,
        now: i64,
    ) -> Result<ChocoChatSession> {
        presence.validate()?;
        store.inspect_session(&token, player_id, now)?;
        if self.online(player_id).is_some() {
            return Err(Error::AlreadyRegistered);
        }
        self.prune_players(now);
        if self.players.len() >= CHAT_PLAYER_CAP && !self.players.contains_key(&player_id) {
            return Err(Error::Capacity);
        }
        let generation = NEXT_SESSION
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
            .map_err(|_| Error::Capacity)?;
        let handle = ChocoChatSession {
            player_id,
            generation,
        };
        self.players.entry(player_id).or_default().online = Some(Online {
            handle,
            token,
            presence,
            service_channel: None,
        });
        Ok(handle)
    }

    pub fn update_presence(
        &mut self,
        session: ChocoChatSession,
        presence: ChocoChatPresence,
    ) -> Result<()> {
        presence.validate()?;
        self.players
            .get_mut(&session.player_id)
            .and_then(|state| state.online.as_mut())
            .filter(|online| online.handle == session)
            .ok_or(Error::NotRegistered)?
            .presence = presence;
        Ok(())
    }

    /// Server-owned channel assignment. Refresh on channel changes and pass
    /// `None` on leave. Reconnections start without a channel assignment.
    pub fn update_f5_service_channel(
        &mut self,
        session: ChocoChatSession,
        channel: Option<NonZeroU32>,
    ) -> Result<()> {
        self.players
            .get_mut(&session.player_id)
            .and_then(|state| state.online.as_mut())
            .filter(|online| online.handle == session)
            .ok_or(Error::NotRegistered)?
            .service_channel = channel;
        Ok(())
    }

    pub fn unregister_session(&mut self, session: ChocoChatSession) -> Result<()> {
        let state = self
            .players
            .get_mut(&session.player_id)
            .ok_or(Error::NotRegistered)?;
        if !state
            .online
            .as_ref()
            .is_some_and(|online| online.handle == session)
        {
            return Err(Error::NotRegistered);
        }
        state.online = None;
        if !state.has_recent_sends() {
            self.players.remove(&session.player_id);
        }
        Ok(())
    }

    fn online(&self, player_id: i64) -> Option<&Online> {
        self.players.get(&player_id)?.online.as_ref()
    }

    fn authenticated(
        &self,
        store: &mut Store,
        session: Option<ChocoChatSession>,
        now_utc: i64,
    ) -> Result<&Online> {
        let session = session.ok_or(Error::Unauthenticated)?;
        let online = self
            .online(session.player_id)
            .filter(|online| online.handle == session)
            .ok_or(Error::NotRegistered)?;
        store.inspect_session(&online.token, session.player_id, now_utc)?;
        Ok(online)
    }

    fn whisper_target(
        &self,
        store: &mut Store,
        sender_id: i64,
        target_id: i64,
        now_utc: i64,
    ) -> Result<ChocoChatSession> {
        let target = self
            .online(target_id)
            .filter(|_| target_id != sender_id)
            .ok_or(Error::TargetUnavailable)?;
        if !target.is_active(store, now_utc)? {
            return Err(Error::TargetUnavailable);
        }
        Ok(target.handle)
    }

    fn guild_audience(store: &mut Store, sender: &Online, now_utc: i64) -> Result<Audience> {
        let (guild, members) = store
            .guild_for_player(sender.handle.player_id, now_utc)?
            .ok_or(Error::NoGuild)?;
        if sender.presence.guild_id != Some(guild.id) {
            return Err(Error::NoGuild);
        }
        Ok(Audience::Guild(guild.id, members))
    }

    fn broadcast_recipients(
        &self,
        store: &mut Store,
        sender_id: i64,
        audience: &Audience,
        filter_blocks: bool,
        now_utc: i64,
    ) -> Result<Vec<ChocoChatSession>> {
        let mut recipients = Vec::new();
        for (&id, state) in &self.players {
            let Some(target) = state.online.as_ref().filter(|_| id != sender_id) else {
                continue;
            };
            let in_audience = match audience {
                Audience::Everyone => true,
                Audience::Team(room, team) => {
                    target.presence.room_id.zip(target.presence.team_id) == Some((*room, *team))
                }
                Audience::Guild(guild, members) => {
                    target.presence.guild_id == Some(*guild)
                        && members.iter().any(|member| member.player_id == id)
                }
                Audience::ServiceChannel(channel) => target.service_channel == Some(*channel),
            };
            if in_audience
                && target.is_active(store, now_utc)?
                && (!filter_blocks || store.can_receive_public_chat(sender_id, id, now_utc)?)
            {
                recipients.push(target.handle);
            }
        }
        Ok(recipients)
    }

    fn record_f5_abuse(&mut self, event: F5AbuseEvent) {
        if self.f5_abuse_events.len() >= F5_ABUSE_EVENT_CAP {
            self.f5_abuse_events.pop_front();
        }
        self.f5_abuse_events.push_back(event);
    }

    pub fn f5_abuse_events(&self) -> impl Iterator<Item = &F5AbuseEvent> {
        self.f5_abuse_events.iter()
    }

    fn f5_enabled(&self, kind: F5MessageKind) -> bool {
        match kind {
            F5MessageKind::Whisper => self.f5_policy.whisper,
            F5MessageKind::Guild => self.f5_policy.guild_chat,
            F5MessageKind::Megaphone(scope) => {
                self.f5_policy.megaphone.is_some_and(|policy| match scope {
                    F5MegaphoneScope::Global => policy.global,
                    F5MegaphoneScope::Channel => policy.channel,
                })
            }
            F5MessageKind::OfflineNote => self.f5_policy.note_inbox_cap.is_some(),
        }
    }

    /// Route local F5 messages to eligible registrations. Extended KR packets
    /// cannot enter this API: their framing and opcodes remain unmeasured.
    pub fn route_f5(
        &mut self,
        store: &mut Store,
        session: Option<ChocoChatSession>,
        route: F5ChatRoute,
        message: F5Message<'_>,
        now_utc: i64,
    ) -> F5Result<Vec<ChocoChatSession>> {
        self.send_f5(store, session, F5Destination::Chat(route), message, now_utc)
    }

    /// Authorize one offline note, without sending or storing it. Check the
    /// authoritative count and insert atomically at the storage owner. A full
    /// inbox is reported only after moderation has authorized the attempt,
    /// which consumes a rate slot to bound repeated probes.
    pub fn authorize_f5_note(
        &mut self,
        store: &mut Store,
        session: Option<ChocoChatSession>,
        inbox: F5NoteInbox,
        message: F5Message<'_>,
        now_utc: i64,
    ) -> F5Result<()> {
        self.send_f5(store, session, F5Destination::Note(inbox), message, now_utc)
            .map(|_| ())
    }

    fn send_f5(
        &mut self,
        store: &mut Store,
        session: Option<ChocoChatSession>,
        destination: F5Destination,
        message: F5Message<'_>,
        now_utc: i64,
    ) -> F5Result<Vec<ChocoChatSession>> {
        let (kind, recipient_id) = destination.metadata();
        if !self.f5_enabled(kind) {
            return Err(F5Error::Disabled);
        }
        let sender_id = self
            .authenticated(store, session, now_utc)?
            .handle
            .player_id;
        self.prune_players(now_utc);
        let result = self.check_f5_send(store, sender_id, destination, message, now_utc);
        if let Err(error) = result
            && let Some(reason) = F5AbuseReason::for_error(error)
        {
            let state = &self.players[&sender_id];
            let accepted_in_window = match reason {
                F5AbuseReason::MegaphoneCooldown | F5AbuseReason::MegaphoneQuota => {
                    state.megaphone_sends.0.len()
                }
                _ => state.f5_sends.0.len(),
            };
            self.record_f5_abuse(F5AbuseEvent {
                sender_id,
                recipient_id,
                kind,
                reason,
                submitted_bytes: message.submitted_bytes(),
                accepted_in_window,
                at_utc: now_utc,
            });
        }
        result
    }

    fn check_f5_send(
        &mut self,
        store: &mut Store,
        sender_id: i64,
        destination: F5Destination,
        message: F5Message<'_>,
        now_utc: i64,
    ) -> F5Result<Vec<ChocoChatSession>> {
        let bytes = message.validate()?;
        let state = &self.players[&sender_id];
        let sender = state.online.as_ref().expect("authenticated sender");
        let delivery = match destination {
            F5Destination::Chat(F5ChatRoute::Whisper(target)) => {
                F5Delivery::Direct(self.whisper_target(store, sender_id, target, now_utc)?)
            }
            F5Destination::Chat(F5ChatRoute::Guild) => {
                F5Delivery::Broadcast(Self::guild_audience(store, sender, now_utc)?)
            }
            F5Destination::Chat(F5ChatRoute::Megaphone(scope)) => {
                let audience = match scope {
                    F5MegaphoneScope::Global => Audience::Everyone,
                    F5MegaphoneScope::Channel => Audience::ServiceChannel(
                        sender.service_channel.ok_or(F5Error::NoServiceChannel)?,
                    ),
                };
                let policy = self.f5_policy.megaphone.ok_or(F5Error::Disabled)?;
                let history = &state.megaphone_sends.0;
                if history
                    .back()
                    .is_some_and(|&at| now_utc - at < policy.cooldown_seconds)
                {
                    return Err(F5Error::MegaphoneCooldown);
                }
                if history.len() >= policy.quota {
                    return Err(F5Error::MegaphoneQuota);
                }
                F5Delivery::Broadcast(audience)
            }
            F5Destination::Note(inbox) => {
                if inbox.recipient_id <= 0 || inbox.recipient_id == sender_id {
                    return Err(Error::TargetUnavailable.into());
                }
                F5Delivery::Note(inbox)
            }
        };
        if state.f5_sends.0.len() >= CHAT_WINDOW_CAP as usize {
            return Err(Error::RateLimited.into());
        }
        check_permission(store, sender_id, destination.metadata().1, bytes, now_utc)?;

        let state = self
            .players
            .get_mut(&sender_id)
            .expect("authenticated sender");
        state.f5_sends.0.push_back(now_utc);
        if matches!(destination, F5Destination::Chat(F5ChatRoute::Megaphone(_))) {
            state.megaphone_sends.0.push_back(now_utc);
        }
        match delivery {
            F5Delivery::Direct(target) => Ok(vec![target]),
            F5Delivery::Broadcast(audience) => {
                Ok(self.broadcast_recipients(store, sender_id, &audience, true, now_utc)?)
            }
            F5Delivery::Note(inbox) => {
                let cap = self.f5_policy.note_inbox_cap.ok_or(F5Error::Disabled)?;
                if inbox.messages >= usize::from(cap.get()) {
                    return Err(F5Error::InboxFull);
                }
                Ok(Vec::new())
            }
        }
    }

    pub fn validate_f5_couple(&self, action: F5CoupleAction) -> F5Result<()> {
        if !self.f5_policy.couple {
            return Err(F5Error::Disabled);
        }
        match action {
            F5CoupleAction::Pair {
                sender_id,
                recipient_id,
                sender_partner,
                recipient_partner,
            } => {
                if sender_id <= 0 || recipient_id <= 0 || sender_id == recipient_id {
                    return Err(Error::TargetUnavailable.into());
                }
                if sender_partner.is_some() || recipient_partner.is_some() {
                    return Err(F5Error::PartnerOccupied);
                }
                Ok(())
            }
            F5CoupleAction::EquipRing => Err(F5Error::UnmeasuredRule(F5UnmeasuredRule::CoupleRing)),
            F5CoupleAction::AwardExp => Err(F5Error::UnmeasuredRule(F5UnmeasuredRule::CoupleExp)),
            F5CoupleAction::AwardLucci => {
                Err(F5Error::UnmeasuredRule(F5UnmeasuredRule::CoupleLucci))
            }
        }
    }

    pub fn validate_f5_plaza(&self, zone: F5PlazaZone, action: F5PlazaAction) -> F5Result<()> {
        let limits = self
            .f5_policy
            .plaza_zones
            .get(&zone)
            .ok_or(F5Error::Disabled)?;
        match action {
            F5PlazaAction::Enter { occupants } => {
                if occupants >= usize::from(limits.max_occupants.get()) {
                    Err(F5Error::PlazaFull)
                } else {
                    Ok(())
                }
            }
            F5PlazaAction::Sit {
                present,
                seat,
                taken,
                already_seated,
            } => {
                if !present {
                    return Err(F5Error::NotInPlaza);
                }
                if already_seated || seat >= limits.seats {
                    return Err(F5Error::InvalidSeat);
                }
                if taken {
                    return Err(F5Error::SeatUnavailable);
                }
                Ok(())
            }
            F5PlazaAction::ClaimCafeReward => {
                Err(F5Error::UnmeasuredRule(F5UnmeasuredRule::CafeReward))
            }
        }
    }

    pub fn reject_f5_opcode(&self, opcode: u16) -> F5Result<()> {
        Err(F5Error::UnmeasuredKrOpcode(opcode))
    }

    /// Returns only eligible live registrations, in player ID order, without
    /// an echo to the sender. The owner uses these decisions for the current
    /// send and never caches them. Store errors produce no delivery list.
    pub fn route(
        &mut self,
        store: &mut Store,
        session: Option<ChocoChatSession>,
        channel: ChocoChatChannel,
        message: &str,
        now: i64,
    ) -> Result<Vec<ChocoChatSession>> {
        let sender = self.authenticated(store, session, now)?;
        let sender_id = sender.handle.player_id;

        let bytes = message.len();
        if !(1..=CHAT_MAX_BYTES).contains(&bytes) {
            // Schema 9 can record the actual invalid byte count (or a mute).
            check_permission(store, sender_id, None, bytes, now)?;
            return Err(Error::InvalidLength);
        }
        if !valid_chat_text(message) {
            return Err(Error::InvalidText);
        }

        if let ChocoChatChannel::Whisper(target_id) = channel {
            let target = self.whisper_target(store, sender_id, target_id, now)?;
            check_permission(store, sender_id, Some(target_id), bytes, now)?;
            return Ok(vec![target]);
        }

        let rolling = matches!(channel, ChocoChatChannel::All | ChocoChatChannel::Shout);
        let cutoff = now - CHAT_WINDOW_SECONDS;
        if rolling
            && self.players[&sender_id]
                .broadcasts
                .iter()
                .filter(|&&time| time > cutoff)
                .count()
                >= CHAT_WINDOW_CAP as usize
        {
            // Schema 9 only audits its own fixed-window denials.
            return Err(Error::RateLimited);
        }

        let audience = match channel {
            ChocoChatChannel::Guild => Self::guild_audience(store, sender, now)?,
            ChocoChatChannel::Team => {
                let (room, team) = sender
                    .presence
                    .room_id
                    .zip(sender.presence.team_id)
                    .ok_or(Error::NoTeam)?;
                Audience::Team(room, team)
            }
            _ => Audience::Everyone,
        };
        check_permission(store, sender_id, None, bytes, now)?;
        let recipients = self.broadcast_recipients(
            store,
            sender_id,
            &audience,
            channel != ChocoChatChannel::Guild,
            now,
        )?;
        if rolling {
            let history = &mut self
                .players
                .get_mut(&sender_id)
                .expect("registered sender")
                .broadcasts;
            history.retain(|&time| time > cutoff);
            history.push(now);
        }
        Ok(recipients)
    }
}

impl Online {
    fn is_active(&self, store: &mut Store, now: i64) -> Result<bool> {
        match store.inspect_session(&self.token, self.handle.player_id, now) {
            Ok(_) => Ok(true),
            Err(accounts::Error::InvalidSession | accounts::Error::InvalidCredentials) => Ok(false),
            Err(error) => Err(error.into()),
        }
    }
}

fn check_permission(
    store: &mut Store,
    sender: i64,
    target: Option<i64>,
    bytes: usize,
    now: i64,
) -> Result<()> {
    match store.check_chat_permission(sender, target, bytes, now)? {
        ChatPermission::Allowed => Ok(()),
        ChatPermission::Blocked => Err(Error::Blocked),
        ChatPermission::Muted => Err(Error::Muted),
        ChatPermission::InvalidLength => Err(Error::InvalidLength),
        ChatPermission::RateLimited => Err(Error::RateLimited),
    }
}

#[cfg(test)]
mod tests {
    use super::{ChocoChatChannel::*, *};
    use crate::{accounts::ChatMuteReason, platform::provision_private_dir};
    use std::{fs, path::PathBuf};

    const NOW: i64 = 100;
    const HELLO: &str = "hello";

    struct TempDir(PathBuf);

    impl TempDir {
        fn new() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let path = std::env::temp_dir().join(format!(
                "kr-choco-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            provision_private_dir(&path).unwrap();
            Self(path)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    struct Fixture {
        router: ChocoChatRouter,
        store: Store,
        _dir: TempDir,
    }

    impl Fixture {
        fn new() -> Self {
            Self::with_policy(F5SocialPolicy::default())
        }

        fn with_policy(policy: F5SocialPolicy) -> Self {
            let dir = TempDir::new();
            let mut store = Store::open(&dir.0.join("chat.sqlite")).unwrap();
            store
                .begin_instance(
                    &accounts::Admin {
                        actor: "test",
                        reason: "local chat policy test",
                    },
                    NOW,
                )
                .unwrap();
            Self {
                router: ChocoChatRouter::with_f5_policy(policy).unwrap(),
                store,
                _dir: dir,
            }
        }

        fn online(&mut self, presence: ChocoChatPresence) -> (ChocoChatSession, Arc<Token>) {
            let grant = self
                .store
                .trial_local(1, 100, None, NOW)
                .unwrap()
                .pop()
                .unwrap();
            let token = Arc::new(grant.token);
            let session = self
                .router
                .register_session(
                    &mut self.store,
                    grant.player.player_id,
                    token.clone(),
                    presence,
                    NOW,
                )
                .unwrap();
            (session, token)
        }

        fn send(
            &mut self,
            sender: ChocoChatSession,
            channel: ChocoChatChannel,
            now: i64,
        ) -> Result<Vec<ChocoChatSession>> {
            self.router
                .route(&mut self.store, Some(sender), channel, HELLO, now)
        }
    }

    impl Fixture {
        fn send_f5(
            &mut self,
            sender: ChocoChatSession,
            route: F5ChatRoute,
            now_utc: i64,
        ) -> F5Result<Vec<ChocoChatSession>> {
            self.router.route_f5(
                &mut self.store,
                Some(sender),
                route,
                F5Message::Utf8(HELLO),
                now_utc,
            )
        }

        fn note(
            &mut self,
            sender: ChocoChatSession,
            recipient_id: i64,
            messages: usize,
            now_utc: i64,
        ) -> F5Result<()> {
            self.router.authorize_f5_note(
                &mut self.store,
                Some(sender),
                F5NoteInbox {
                    recipient_id,
                    messages,
                    unread: messages,
                },
                F5Message::Utf8(HELLO),
                now_utc,
            )
        }
    }

    const GLOBAL: F5ChatRoute = F5ChatRoute::Megaphone(F5MegaphoneScope::Global);
    const CHANNEL: F5ChatRoute = F5ChatRoute::Megaphone(F5MegaphoneScope::Channel);
    const PLAZAS: [F5PlazaZone; 6] = [
        F5PlazaZone::LobbySquare,
        F5PlazaZone::LobbyCaffe,
        F5PlazaZone::LobbyClothing,
        F5PlazaZone::LobbyJewelry,
        F5PlazaZone::LobbyPet,
        F5PlazaZone::Square,
    ];
    fn proposal(
        sender_id: i64,
        recipient_id: i64,
        sender_partner: Option<i64>,
        recipient_partner: Option<i64>,
    ) -> F5CoupleAction {
        F5CoupleAction::Pair {
            sender_id,
            recipient_id,
            sender_partner,
            recipient_partner,
        }
    }

    fn megaphone() -> F5MegaphonePolicy {
        F5MegaphonePolicy {
            global: true,
            channel: true,
            cooldown_seconds: 3,
            window_seconds: 20,
            quota: 2,
        }
    }

    #[test]
    fn extended_paths_default_off_and_wire_stays_unmeasured() {
        let mut f = Fixture::new();
        f.router = ChocoChatRouter::default();
        let (sender, _) = f.online(Default::default());
        let (peer, _) = f.online(Default::default());
        for route in [
            F5ChatRoute::Whisper(peer.player_id()),
            F5ChatRoute::Guild,
            GLOBAL,
            CHANNEL,
        ] {
            assert_eq!(f.send_f5(sender, route, NOW), Err(F5Error::Disabled));
        }
        assert_eq!(
            f.note(sender, peer.player_id(), 0, NOW),
            Err(F5Error::Disabled)
        );
        assert_eq!(
            f.router.validate_f5_couple(proposal(1, 2, None, None)),
            Err(F5Error::Disabled)
        );
        for zone in PLAZAS {
            assert_eq!(
                f.router
                    .validate_f5_plaza(zone, F5PlazaAction::Enter { occupants: 0 }),
                Err(F5Error::Disabled)
            );
        }
        let enabled = ChocoChatRouter::with_f5_policy(F5SocialPolicy {
            whisper: true,
            couple: true,
            ..Default::default()
        })
        .unwrap();
        for opcode in [0, 474, 640, 670, u16::MAX] {
            assert_eq!(
                f.router.reject_f5_opcode(opcode),
                Err(F5Error::UnmeasuredKrOpcode(opcode))
            );
            assert_eq!(
                enabled.reject_f5_opcode(opcode),
                Err(F5Error::UnmeasuredKrOpcode(opcode))
            );
        }
        assert_eq!(f.router.f5_abuse_events().count(), 0);
        assert_eq!(
            f.send(sender, Whisper(peer.player_id()), NOW),
            Ok(vec![peer])
        );
    }

    #[test]
    fn extended_chat_requires_live_sessions_and_both_text_lengths() {
        let mut f = Fixture::with_policy(F5SocialPolicy {
            whisper: true,
            ..Default::default()
        });
        let (sender, _) = f.online(Default::default());
        let (peer, peer_token) = f.online(Default::default());
        let route = F5ChatRoute::Whisper(peer.player_id());
        let at_limit = "한".repeat(CHAT_MAX_BYTES / 3) + &"a".repeat(CHAT_MAX_BYTES % 3);
        let over_utf8 = at_limit.clone() + "a";
        let short_949x = text949x::encode(&over_utf8).unwrap();
        let cases = [
            (F5Message::Utf8(""), Err(Error::InvalidLength)),
            (F5Message::Utf8(&over_utf8), Err(Error::InvalidLength)),
            (F5Message::Text949x(&short_949x), Err(Error::InvalidLength)),
            (
                F5Message::Text949x(&[b'a'; CHAT_MAX_BYTES + 1]),
                Err(Error::InvalidLength),
            ),
            (F5Message::Utf8("😀"), Err(Error::InvalidText)),
            (F5Message::Utf8(" \u{2003}"), Err(Error::InvalidText)),
            (F5Message::Text949x(b"a\0b"), Err(Error::InvalidText)),
            (F5Message::Text949x(&[0xc7]), Err(Error::InvalidText)),
            (F5Message::Utf8(&at_limit), Ok(vec![peer])),
            (
                F5Message::Text949x(&[b'V', b'i', 0xd2, 0x84, b't']),
                Ok(vec![peer]),
            ),
            (F5Message::Text949x(&[b'a'; CHAT_MAX_BYTES]), Ok(vec![peer])),
        ];
        assert!(short_949x.len() < CHAT_MAX_BYTES);
        assert_eq!(
            f.router
                .route_f5(&mut f.store, None, route, F5Message::Utf8(HELLO), NOW),
            Err(F5Error::Chat(Error::Unauthenticated))
        );
        for (body, expected) in cases {
            assert_eq!(
                f.router
                    .route_f5(&mut f.store, Some(sender), route, body, NOW),
                expected.map_err(F5Error::Chat)
            );
        }
        f.store.logout(&peer_token, peer.player_id(), NOW).unwrap();
        assert_eq!(
            f.send_f5(sender, route, NOW),
            Err(F5Error::Chat(Error::TargetUnavailable))
        );
        assert_eq!(
            f.send_f5(peer, F5ChatRoute::Whisper(sender.player_id()), NOW),
            Err(F5Error::Chat(Error::Unauthenticated))
        );
        assert_eq!(
            f.send_f5(sender, F5ChatRoute::Guild, NOW),
            Err(F5Error::Disabled)
        );
    }

    #[test]
    fn guild_and_whisper_share_rate_limit_and_honor_live_moderation() {
        let mut f = Fixture::with_policy(F5SocialPolicy {
            whisper: true,
            guild_chat: true,
            ..Default::default()
        });
        let (sender, _) = f.online(Default::default());
        let (blocked, _) = f.online(Default::default());
        let (peer, _) = f.online(Default::default());
        let guild = f
            .store
            .create_guild(sender.player_id(), "Guard Guild", NOW)
            .unwrap();
        for session in [blocked, peer] {
            f.store
                .join_guild(session.player_id(), guild.id, NOW)
                .unwrap();
        }
        assert_eq!(
            f.send_f5(sender, F5ChatRoute::Guild, NOW),
            Err(F5Error::Chat(Error::NoGuild))
        );
        for session in [sender, blocked, peer] {
            f.router
                .update_presence(
                    session,
                    ChocoChatPresence {
                        guild_id: Some(guild.id),
                        ..Default::default()
                    },
                )
                .unwrap();
        }
        f.store
            .block_player(blocked.player_id(), sender.player_id(), NOW)
            .unwrap();
        assert_eq!(
            f.send_f5(sender, F5ChatRoute::Whisper(blocked.player_id()), NOW),
            Err(F5Error::Chat(Error::Blocked))
        );
        let expires = f
            .store
            .mute_social_player(sender.player_id(), 10, ChatMuteReason::Moderation, NOW)
            .unwrap();
        assert_eq!(
            f.send_f5(sender, F5ChatRoute::Guild, expires - 1),
            Err(F5Error::Chat(Error::Muted))
        );
        assert_eq!(
            f.send_f5(sender, F5ChatRoute::Whisper(peer.player_id()), expires - 1),
            Err(F5Error::Chat(Error::Muted))
        );
        let last = expires + CHAT_WINDOW_SECONDS - 1;
        for n in 0..CHAT_WINDOW_CAP {
            let route = if n % 2 == 0 {
                F5ChatRoute::Guild
            } else {
                F5ChatRoute::Whisper(peer.player_id())
            };
            assert_eq!(f.send_f5(sender, route, last), Ok(vec![peer]));
        }
        assert_eq!(
            f.send_f5(sender, F5ChatRoute::Guild, last + 1),
            Err(F5Error::Chat(Error::RateLimited))
        );
        assert_eq!(
            f.send_f5(peer, F5ChatRoute::Whisper(sender.player_id()), last + 1),
            Ok(vec![sender])
        );
        assert_eq!(
            f.send_f5(sender, F5ChatRoute::Guild, last + CHAT_WINDOW_SECONDS),
            Ok(vec![peer])
        );
        assert_eq!(
            f.router
                .f5_abuse_events()
                .map(|event| event.reason)
                .collect::<Vec<_>>(),
            [
                F5AbuseReason::Blocked,
                F5AbuseReason::Muted,
                F5AbuseReason::Muted,
                F5AbuseReason::RateLimited,
            ]
        );
        assert_eq!(
            f.router
                .f5_abuse_events()
                .last()
                .unwrap()
                .accepted_in_window,
            CHAT_WINDOW_CAP as usize
        );
    }

    #[test]
    fn megaphone_uses_current_channel_and_a_shared_per_account_window() {
        let mut f = Fixture::with_policy(F5SocialPolicy {
            megaphone: Some(megaphone()),
            ..Default::default()
        });
        let (sender, token) = f.online(Default::default());
        let (peer, _) = f.online(Default::default());
        let (other, _) = f.online(Default::default());
        let (blocked, _) = f.online(Default::default());
        assert_eq!(
            f.send_f5(sender, CHANNEL, NOW),
            Err(F5Error::NoServiceChannel)
        );
        for session in [sender, peer, blocked] {
            f.router
                .update_f5_service_channel(session, NonZeroU32::new(1))
                .unwrap();
        }
        f.router
            .update_f5_service_channel(other, NonZeroU32::new(2))
            .unwrap();
        f.store
            .block_player(blocked.player_id(), sender.player_id(), NOW)
            .unwrap();
        assert_eq!(f.send_f5(sender, CHANNEL, NOW), Ok(vec![peer]));
        assert_eq!(
            f.send_f5(sender, GLOBAL, NOW + 2),
            Err(F5Error::MegaphoneCooldown)
        );
        assert_eq!(f.send_f5(sender, GLOBAL, NOW + 3), Ok(vec![peer, other]));
        assert_eq!(f.send_f5(other, CHANNEL, NOW + 3), Ok(vec![]));
        assert_eq!(
            f.send_f5(sender, CHANNEL, NOW + 6),
            Err(F5Error::MegaphoneQuota)
        );

        f.router.unregister_session(sender).unwrap();
        let reconnected = f
            .router
            .register_session(
                &mut f.store,
                sender.player_id(),
                token,
                Default::default(),
                NOW + 19,
            )
            .unwrap();
        assert_eq!(
            f.send_f5(sender, GLOBAL, NOW + 19),
            Err(F5Error::Chat(Error::NotRegistered))
        );
        assert_eq!(
            f.send_f5(reconnected, CHANNEL, NOW + 19),
            Err(F5Error::NoServiceChannel)
        );
        assert_eq!(
            f.send_f5(reconnected, GLOBAL, NOW + 19),
            Err(F5Error::MegaphoneQuota)
        );
        f.router
            .update_f5_service_channel(reconnected, NonZeroU32::new(2))
            .unwrap();
        assert_eq!(f.send_f5(reconnected, CHANNEL, NOW + 20), Ok(vec![other]));
        assert_eq!(
            f.send_f5(reconnected, CHANNEL, NOW + 19),
            Err(F5Error::Chat(Error::Store(accounts::Error::ClockRollback)))
        );
        let events = f.router.f5_abuse_events().copied().collect::<Vec<_>>();
        assert_eq!(events.len(), 3);
        assert_eq!(
            events[0].kind,
            F5MessageKind::Megaphone(F5MegaphoneScope::Global)
        );
        assert_eq!(events[0].reason, F5AbuseReason::MegaphoneCooldown);
        assert_eq!(events[0].accepted_in_window, 1);
        assert_eq!(events[2].reason, F5AbuseReason::MegaphoneQuota);
        assert_eq!(events[2].accepted_in_window, 2);
        let expiry = f
            .store
            .mute_social_player(reconnected.player_id(), 10, ChatMuteReason::Spam, NOW + 30)
            .unwrap();
        assert_eq!(
            f.send_f5(reconnected, GLOBAL, expiry - 1),
            Err(F5Error::Chat(Error::Muted))
        );
        assert_eq!(
            f.send_f5(reconnected, GLOBAL, expiry),
            Ok(vec![peer, other])
        );
    }

    #[test]
    fn megaphone_configuration_rejects_unbounded_or_inert_policies() {
        for bad in [
            F5MegaphonePolicy {
                quota: 0,
                ..megaphone()
            },
            F5MegaphonePolicy {
                quota: F5_MAX_QUOTA + 1,
                ..megaphone()
            },
            F5MegaphonePolicy {
                cooldown_seconds: -1,
                ..megaphone()
            },
            F5MegaphonePolicy {
                cooldown_seconds: 21,
                ..megaphone()
            },
            F5MegaphonePolicy {
                window_seconds: i64::MAX,
                ..megaphone()
            },
            F5MegaphonePolicy {
                global: false,
                channel: false,
                ..megaphone()
            },
        ] {
            assert!(matches!(
                ChocoChatRouter::with_f5_policy(F5SocialPolicy {
                    megaphone: Some(bad),
                    ..Default::default()
                }),
                Err(F5Error::InvalidPolicy)
            ));
        }
        let mut f = Fixture::with_policy(F5SocialPolicy {
            megaphone: Some(F5MegaphonePolicy {
                global: false,
                ..megaphone()
            }),
            ..Default::default()
        });
        let (sender, _) = f.online(Default::default());
        assert_eq!(f.send_f5(sender, GLOBAL, NOW), Err(F5Error::Disabled));
        assert_eq!(
            f.send_f5(sender, CHANNEL, NOW),
            Err(F5Error::NoServiceChannel)
        );
    }

    #[test]
    fn offline_notes_check_recipient_blocks_mute_and_inbox_bound() {
        let mut f = Fixture::with_policy(F5SocialPolicy {
            note_inbox_cap: NonZeroU16::new(2),
            ..Default::default()
        });
        let (sender, _) = f.online(Default::default());
        let (recipient, token) = f.online(Default::default());
        let id = recipient.player_id();
        f.router.unregister_session(recipient).unwrap();
        f.store.logout(&token, id, NOW).unwrap();

        assert_eq!(
            f.note(sender, sender.player_id(), 0, NOW),
            Err(F5Error::Chat(Error::TargetUnavailable))
        );
        f.store.block_player(id, sender.player_id(), NOW).unwrap();
        assert_eq!(
            f.note(sender, id, usize::MAX, NOW),
            Err(F5Error::Chat(Error::Blocked))
        );
        f.store.unblock_player(id, sender.player_id(), NOW).unwrap();
        let expiry = f
            .store
            .mute_social_player(sender.player_id(), 10, ChatMuteReason::Spam, NOW)
            .unwrap();
        assert_eq!(
            f.note(sender, id, 0, expiry - 1),
            Err(F5Error::Chat(Error::Muted))
        );
        assert_eq!(f.note(sender, id, 1, expiry), Ok(()));
        assert_eq!(f.note(sender, id, 2, expiry), Err(F5Error::InboxFull));
        assert_eq!(
            f.note(sender, id, usize::MAX, expiry),
            Err(F5Error::InboxFull)
        );
        assert_eq!(f.note(sender, id, 1, expiry), Ok(()));
        assert_eq!(f.note(sender, id, 1, expiry), Ok(()));
        assert_eq!(
            f.note(sender, id, 1, expiry),
            Err(F5Error::Chat(Error::RateLimited))
        );
    }

    #[test]
    fn rolling_state_prunes_expired_offline_accounts_and_audit_is_bounded() {
        let mut f = Fixture::with_policy(F5SocialPolicy {
            whisper: true,
            megaphone: Some(megaphone()),
            ..Default::default()
        });
        let (sender, _) = f.online(Default::default());
        let (peer, _) = f.online(Default::default());
        let (stays, _) = f.online(Default::default());
        f.send_f5(sender, GLOBAL, NOW).unwrap();
        f.send_f5(peer, F5ChatRoute::Whisper(stays.player_id()), NOW)
            .unwrap();
        f.router.unregister_session(sender).unwrap();
        f.router.unregister_session(peer).unwrap();
        assert_eq!(f.router.players.len(), 3);

        for n in 0..F5_ABUSE_EVENT_CAP + 1 {
            let body = if n == 0 { "" } else { "discard this\nbody" };
            assert!(
                f.router
                    .route_f5(
                        &mut f.store,
                        Some(stays),
                        GLOBAL,
                        F5Message::Utf8(body),
                        NOW + 10
                    )
                    .is_err()
            );
        }
        assert_eq!(f.router.players.len(), 2);
        assert!(!f.router.players.contains_key(&peer.player_id()));
        assert!(f.router.players.contains_key(&sender.player_id()));
        let event = F5AbuseEvent {
            sender_id: stays.player_id(),
            recipient_id: None,
            kind: F5MessageKind::Megaphone(F5MegaphoneScope::Global),
            reason: F5AbuseReason::InvalidText,
            submitted_bytes: "discard this\nbody".len(),
            accepted_in_window: 0,
            at_utc: NOW + 10,
        };
        assert_eq!(f.router.f5_abuse_events().count(), F5_ABUSE_EVENT_CAP);
        assert!(f.router.f5_abuse_events().all(|record| *record == event));
        assert!(
            f.store
                .list_chat_abuse(Some(stays.player_id()), None, 10)
                .unwrap()
                .is_empty()
        );
        assert_eq!(f.send_f5(stays, GLOBAL, NOW + 20), Ok(vec![]));
        assert_eq!(f.router.players.len(), 1);
    }

    #[test]
    fn couple_requires_two_free_distinct_partners_and_denies_unmeasured_rewards() {
        let router = ChocoChatRouter::with_f5_policy(F5SocialPolicy {
            couple: true,
            ..Default::default()
        })
        .unwrap();
        assert_eq!(
            router.validate_f5_couple(proposal(1, 2, None, None)),
            Ok(())
        );
        for (sender_partner, recipient_partner) in
            [(Some(3), None), (None, Some(3)), (Some(2), Some(1))]
        {
            assert_eq!(
                router.validate_f5_couple(proposal(1, 2, sender_partner, recipient_partner)),
                Err(F5Error::PartnerOccupied)
            );
        }
        for recipient_id in [-1, 0, 1] {
            assert_eq!(
                router.validate_f5_couple(proposal(1, recipient_id, None, None)),
                Err(F5Error::Chat(Error::TargetUnavailable))
            );
        }
        for (action, rule) in [
            (F5CoupleAction::EquipRing, F5UnmeasuredRule::CoupleRing),
            (F5CoupleAction::AwardExp, F5UnmeasuredRule::CoupleExp),
            (F5CoupleAction::AwardLucci, F5UnmeasuredRule::CoupleLucci),
        ] {
            assert_eq!(
                router.validate_f5_couple(action),
                Err(F5Error::UnmeasuredRule(rule))
            );
        }
    }

    #[test]
    fn each_plaza_family_checks_its_own_occupancy_and_seats() {
        let router = ChocoChatRouter::with_f5_policy(F5SocialPolicy {
            plaza_zones: PLAZAS
                .into_iter()
                .enumerate()
                .map(|(i, zone)| {
                    (
                        zone,
                        F5PlazaLimits {
                            max_occupants: NonZeroU16::new(i as u16 + 1).unwrap(),
                            seats: i as u16,
                        },
                    )
                })
                .collect(),
            ..Default::default()
        })
        .unwrap();
        for (i, zone) in PLAZAS.into_iter().enumerate() {
            assert_eq!(
                router.validate_f5_plaza(zone, F5PlazaAction::Enter { occupants: i }),
                Ok(())
            );
            assert_eq!(
                router.validate_f5_plaza(zone, F5PlazaAction::Enter { occupants: i + 1 }),
                Err(F5Error::PlazaFull)
            );
            let sit = |present, seat, taken, already_seated| {
                router.validate_f5_plaza(
                    zone,
                    F5PlazaAction::Sit {
                        present,
                        seat,
                        taken,
                        already_seated,
                    },
                )
            };
            assert_eq!(sit(true, i as u16, false, false), Err(F5Error::InvalidSeat));
            if i > 0 {
                assert_eq!(sit(true, 0, false, false), Ok(()));
                assert_eq!(sit(false, 0, false, false), Err(F5Error::NotInPlaza));
                assert_eq!(sit(true, 0, true, false), Err(F5Error::SeatUnavailable));
                assert_eq!(sit(true, 0, false, true), Err(F5Error::InvalidSeat));
            }
            assert_eq!(
                router.validate_f5_plaza(zone, F5PlazaAction::ClaimCafeReward),
                Err(F5Error::UnmeasuredRule(F5UnmeasuredRule::CafeReward))
            );
        }
    }

    fn team(room: u32, id: u8) -> ChocoChatPresence {
        ChocoChatPresence {
            room_id: Some(RoomId(room)),
            team_id: Some(id),
            guild_id: None,
        }
    }

    #[test]
    fn mute_denies_every_channel_and_audits_until_expiry() {
        let mut f = Fixture::new();
        let (sender, _) = f.online(team(1, 1));
        let (peer, _) = f.online(team(1, 1));
        let id = sender.player_id();
        let guild = f.store.create_guild(id, "Test Guild", NOW).unwrap();
        f.router
            .update_presence(
                sender,
                ChocoChatPresence {
                    guild_id: Some(guild.id),
                    ..team(1, 1)
                },
            )
            .unwrap();
        let expiry = f
            .store
            .mute_social_player(id, 15, ChatMuteReason::Moderation, NOW)
            .unwrap();

        for channel in [All, Shout, Team, Guild, Whisper(peer.player_id())] {
            assert_eq!(f.send(sender, channel, expiry - 1), Err(Error::Muted));
        }
        let events = f.store.list_chat_abuse(Some(id), None, 10).unwrap();
        assert_eq!(events.len(), 5);
        assert_eq!(events[0].recipient_id, Some(peer.player_id()));
        for event in events {
            assert_eq!(event.reason, "muted");
            assert_eq!(event.message_bytes, HELLO.len() as i64);
            assert_eq!(event.accepted_in_window, 0);
            assert_eq!(event.at_utc, expiry - 1);
        }
        assert_eq!(f.send(sender, All, expiry), Ok(vec![peer]));
    }

    #[test]
    fn one_way_block_denies_whispers_and_filters_both_directions() {
        let mut f = Fixture::new();
        let (a, _) = f.online(team(1, 1));
        let (b, _) = f.online(team(1, 1));
        let (peer, _) = f.online(team(1, 1));
        f.store
            .block_player(b.player_id(), a.player_id(), NOW)
            .unwrap();

        for (sender, blocked) in [(a, b), (b, a)] {
            assert_eq!(
                f.send(sender, Whisper(blocked.player_id()), NOW),
                Err(Error::Blocked)
            );
            for channel in [All, Team, Shout] {
                assert_eq!(f.send(sender, channel, NOW), Ok(vec![peer]));
            }
            assert_eq!(
                f.send(sender, Whisper(peer.player_id()), NOW),
                Ok(vec![peer])
            );
            let events = f
                .store
                .list_chat_abuse(Some(sender.player_id()), None, 10)
                .unwrap();
            assert_eq!(events.len(), 1);
            assert_eq!(events[0].reason, "blocked");
            assert_eq!(events[0].recipient_id, Some(blocked.player_id()));
        }
    }

    #[test]
    fn guild_requires_current_membership_and_online_presence() {
        let mut f = Fixture::new();
        let (leader, _) = f.online(ChocoChatPresence::default());
        let (member, _) = f.online(ChocoChatPresence::default());
        let (outsider, _) = f.online(ChocoChatPresence::default());
        let (offline, _) = f.online(ChocoChatPresence::default());
        let guild = f
            .store
            .create_guild(leader.player_id(), "First Guild", NOW)
            .unwrap();
        for session in [member, offline] {
            assert!(
                f.store
                    .join_guild(session.player_id(), guild.id, NOW)
                    .unwrap()
            );
        }
        f.store
            .create_guild(outsider.player_id(), "Other Guild", NOW)
            .unwrap();
        for session in [leader, member, outsider, offline] {
            f.router
                .update_presence(
                    session,
                    ChocoChatPresence {
                        guild_id: Some(guild.id),
                        ..Default::default()
                    },
                )
                .unwrap();
        }
        f.router.unregister_session(offline).unwrap();

        assert_eq!(f.send(leader, Guild, NOW), Ok(vec![member]));
        assert_eq!(f.send(outsider, Guild, NOW), Err(Error::NoGuild));
        assert!(
            f.store
                .leave_guild(member.player_id(), guild.id, NOW)
                .unwrap()
        );
        assert_eq!(f.send(leader, Guild, NOW), Ok(vec![]));
        assert_eq!(f.send(member, Guild, NOW), Err(Error::NoGuild));
    }

    #[test]
    fn team_requires_both_the_same_room_and_team() {
        let mut f = Fixture::new();
        let (sender, _) = f.online(team(1, 1));
        let (teammate, _) = f.online(team(1, 1));
        let (opponent, _) = f.online(team(1, 2));
        let (other_room, _) = f.online(team(2, 1));
        let (unassigned, _) = f.online(ChocoChatPresence {
            room_id: Some(RoomId(1)),
            ..Default::default()
        });
        assert_eq!(f.send(sender, Team, NOW), Ok(vec![teammate]));
        assert_eq!(f.send(unassigned, Team, NOW), Err(Error::NoTeam));
        assert_eq!(
            f.send(sender, All, NOW),
            Ok(vec![teammate, opponent, other_room, unassigned])
        );
        f.router.update_presence(sender, team(2, 1)).unwrap();
        assert_eq!(f.send(sender, Team, NOW), Ok(vec![other_room]));
    }

    #[test]
    fn public_rate_cap_slides_across_store_buckets_and_reconnects() {
        let mut f = Fixture::new();
        let (sender, token) = f.online(team(1, 1));
        let (peer, _) = f.online(team(1, 1));
        let last = NOW + CHAT_WINDOW_SECONDS - 1;
        for n in 0..CHAT_WINDOW_CAP {
            assert_eq!(
                f.send(sender, if n % 2 == 0 { All } else { Shout }, last),
                Ok(vec![peer])
            );
        }
        assert_eq!(f.send(sender, Team, last), Err(Error::RateLimited));
        let events = f
            .store
            .list_chat_abuse(Some(sender.player_id()), None, 10)
            .unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].reason, "rate_limited");
        assert_eq!(events[0].accepted_in_window, CHAT_WINDOW_CAP);

        assert_eq!(f.send(sender, All, last + 1), Err(Error::RateLimited));
        assert_eq!(f.send(peer, Shout, last + 1), Ok(vec![sender]));
        f.router.unregister_session(sender).unwrap();
        let reconnected = f
            .router
            .register_session(
                &mut f.store,
                sender.player_id(),
                token,
                team(1, 1),
                last + 2,
            )
            .unwrap();
        assert_eq!(f.send(sender, All, last + 2), Err(Error::NotRegistered));
        assert_eq!(
            f.send(reconnected, Shout, last + CHAT_WINDOW_SECONDS - 1),
            Err(Error::RateLimited)
        );
        assert_eq!(
            f.send(reconnected, All, last + CHAT_WINDOW_SECONDS),
            Ok(vec![peer])
        );
    }

    #[test]
    fn validates_byte_length_and_text_before_any_delivery() {
        let mut f = Fixture::new();
        let (sender, _) = f.online(ChocoChatPresence::default());
        let (peer, _) = f.online(ChocoChatPresence::default());
        let at_limit = "한".repeat(CHAT_MAX_BYTES / 3) + &"a".repeat(CHAT_MAX_BYTES % 3);
        let oversize = at_limit.clone() + "한";
        let mut route = |msg| f.router.route(&mut f.store, Some(sender), All, msg, NOW);
        assert_eq!(route(""), Err(Error::InvalidLength));
        assert_eq!(route(&oversize), Err(Error::InvalidLength));
        for msg in [" \u{2003}", "x\0y", "x\ny", "x\ty", "x\u{85}y"] {
            assert_eq!(route(msg), Err(Error::InvalidText));
        }
        assert_eq!(route(&at_limit), Ok(vec![peer]));

        let events = f
            .store
            .list_chat_abuse(Some(sender.player_id()), None, 10)
            .unwrap();
        assert_eq!(events.len(), 2);
        for (event, length) in events.iter().zip([oversize.len(), 0]) {
            assert_eq!(event.reason, "invalid_length");
            assert_eq!(event.message_bytes, length as i64);
            assert_eq!(event.accepted_in_window, 0);
        }
    }

    #[test]
    fn unauthenticated_stale_and_revoked_contexts_get_no_route() {
        let mut f = Fixture::new();
        let presence = ChocoChatPresence::default();
        let (sender, token) = f.online(presence);
        let (peer, peer_token) = f.online(presence);
        let (offline, _) = f.online(presence);
        assert_eq!(
            f.router.route(&mut f.store, None, All, HELLO, NOW),
            Err(Error::Unauthenticated)
        );
        assert_eq!(
            f.router.register_session(
                &mut f.store,
                sender.player_id(),
                peer_token.clone(),
                presence,
                NOW
            ),
            Err(Error::Unauthenticated)
        );
        assert_eq!(
            f.router
                .register_session(&mut f.store, sender.player_id(), token, presence, NOW),
            Err(Error::AlreadyRegistered)
        );
        assert_eq!(
            f.send(sender, Whisper(sender.player_id()), NOW),
            Err(Error::TargetUnavailable)
        );

        f.router.unregister_session(offline).unwrap();
        assert_eq!(f.send(offline, All, NOW), Err(Error::NotRegistered));
        assert_eq!(
            f.send(sender, Whisper(offline.player_id()), NOW),
            Err(Error::TargetUnavailable)
        );
        f.store.logout(&peer_token, peer.player_id(), NOW).unwrap();
        assert_eq!(f.send(peer, All, NOW), Err(Error::Unauthenticated));
        assert_eq!(
            f.send(sender, Whisper(peer.player_id()), NOW),
            Err(Error::TargetUnavailable)
        );
        assert_eq!(f.send(sender, All, NOW), Ok(vec![]));
    }

    #[test]
    fn store_failure_denies_instead_of_assuming_permission() {
        let mut f = Fixture::new();
        let (sender, _) = f.online(ChocoChatPresence::default());
        let (peer, _) = f.online(ChocoChatPresence::default());
        assert_eq!(
            f.send(sender, All, NOW - 1),
            Err(Error::Store(accounts::Error::ClockRollback))
        );
        assert_eq!(f.send(sender, All, NOW), Ok(vec![peer]));
    }
}
