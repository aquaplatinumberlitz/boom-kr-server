//! Local F5-6 routing policy. Channel names are logical; no CHOCO wire format
//! or native client behavior is assumed. Run this synchronous scaffold with
//! its Schema 9 Store on a single bounded owner, off the async I/O loop.

use crate::{
    accounts::{
        self, CHAT_MAX_BYTES, CHAT_WINDOW_CAP, CHAT_WINDOW_SECONDS, ChatPermission, Store, Token,
    },
    room_registry::RoomId,
};
use std::{
    collections::BTreeMap,
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
}

#[derive(Default)]
struct PlayerState {
    online: Option<Online>,
    broadcasts: Vec<i64>,
}

#[derive(Default)]
pub struct ChocoChatRouter {
    players: BTreeMap<i64, PlayerState>,
}

impl ChocoChatRouter {
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
        let cutoff = now - CHAT_WINDOW_SECONDS;
        self.players.retain(|_, state| {
            state.online.is_some() || state.broadcasts.last().is_some_and(|&time| time > cutoff)
        });
        if self.players.len() == CHAT_PLAYER_CAP && !self.players.contains_key(&player_id) {
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
        if state.broadcasts.is_empty() {
            self.players.remove(&session.player_id);
        }
        Ok(())
    }

    fn online(&self, player_id: i64) -> Option<&Online> {
        self.players.get(&player_id)?.online.as_ref()
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
        let session = session.ok_or(Error::Unauthenticated)?;
        let sender_id = session.player_id;
        let sender = self
            .online(sender_id)
            .filter(|online| online.handle == session)
            .ok_or(Error::NotRegistered)?;
        store.inspect_session(&sender.token, sender_id, now)?;

        let bytes = message.len();
        if !(1..=CHAT_MAX_BYTES).contains(&bytes) {
            // Schema 9 can record the actual invalid byte count (or a mute).
            check_permission(store, sender_id, None, bytes, now)?;
            return Err(Error::InvalidLength);
        }
        if message.chars().any(char::is_control) || message.trim().is_empty() {
            return Err(Error::InvalidText);
        }

        if let ChocoChatChannel::Whisper(target_id) = channel {
            let target = self
                .online(target_id)
                .filter(|_| target_id != sender_id)
                .ok_or(Error::TargetUnavailable)?;
            if !target.is_active(store, now)? {
                return Err(Error::TargetUnavailable);
            }
            check_permission(store, sender_id, Some(target_id), bytes, now)?;
            return Ok(vec![target.handle]);
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

        let guild = if channel == ChocoChatChannel::Guild {
            let (guild, members) = store
                .guild_for_player(sender_id, now)?
                .ok_or(Error::NoGuild)?;
            if sender.presence.guild_id != Some(guild.id) {
                return Err(Error::NoGuild);
            }
            Some((guild.id, members))
        } else {
            None
        };
        let team = if channel == ChocoChatChannel::Team {
            Some(
                sender
                    .presence
                    .room_id
                    .zip(sender.presence.team_id)
                    .ok_or(Error::NoTeam)?,
            )
        } else {
            None
        };

        check_permission(store, sender_id, None, bytes, now)?;
        let mut recipients = Vec::new();
        for (&id, state) in &self.players {
            let Some(target) = state.online.as_ref().filter(|_| id != sender_id) else {
                continue;
            };
            if let Some((guild_id, members)) = &guild
                && (target.presence.guild_id != Some(*guild_id)
                    || !members.iter().any(|member| member.player_id == id))
            {
                continue;
            }
            if team.is_some_and(|pair| {
                target.presence.room_id.zip(target.presence.team_id) != Some(pair)
            }) {
                continue;
            }
            if target.is_active(store, now)?
                && (guild.is_some() || store.can_receive_public_chat(sender_id, id, now)?)
            {
                recipients.push(target.handle);
            }
        }
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
                router: ChocoChatRouter::default(),
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
