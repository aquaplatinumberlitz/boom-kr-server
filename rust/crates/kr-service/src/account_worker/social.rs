//! Schema 9 service seam. Synchronous Store calls run only on the normal
//! `kr-account-db` worker. Callers use nonblocking admission and await a reply.
//! No KR wire adapter; no message body enters the queue.

use super::economy::AccountService;
use super::{Client, Command, Error, Pending, Result};
use crate::accounts::{
    self, ChatMuteReason, ChatPermission, FriendRequest, Guild, GuildLeaderAction, GuildMember,
    RankingEntry, SocialMute, Store, Token,
};
use crate::choco_chat::{ChocoChatRouter, F5PlazaZone, F5SocialPolicy};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, OnceLock, RwLock, Weak},
    time::Instant,
};
use tokio::sync::oneshot;

#[cfg(test)]
mod tests;

/// Actions scoped to one actor, selected by trusted service code. The IDs in
/// these variants are targets, never the identity of the calling player.
pub enum SocialAction {
    SendFriendRequest(i64),
    AcceptFriendRequest(i64),
    DeclineFriendRequest(i64),
    RespondFriendRequest {
        requester: i64,
        accept: bool,
    },
    CancelFriendRequest(i64),
    RemoveFriend(i64),
    BlockPlayer(i64),
    UnblockPlayer(i64),
    ListFriends,
    ListBlocks,
    ListFriendRequests,
    /// Read the active mute without reserving a send slot or recording a denial.
    MuteStatus,
    /// Only Allowed grants a send slot. Denials record metadata in Schema 9.
    CheckChatPermission {
        recipient: Option<i64>,
        message_bytes: usize,
    },
    /// Broadcast delivery filter. Does not consume another send slot.
    CanReceivePublicChat(i64),
    CreateGuild(String),
    JoinGuild(i64),
    LeaveGuild(i64),
    /// Strict web self-service leave. Leaders and non-members fail closed.
    LeaveGuildSelfService(i64),
    DisbandGuild(i64),
    SetGuildOfficer {
        guild_id: i64,
        player: i64,
        officer: bool,
    },
    TransferGuildLeadership {
        guild_id: i64,
        successor: i64,
    },
    ManageGuild {
        expected_guild: Option<i64>,
        action: GuildLeaderAction,
    },
    GuildForPlayer,
    ListGuildMembers(i64),
    /// Stable ordering on unchanged data; requests do not share a snapshot.
    Ranking {
        offset: u32,
        limit: u32,
    },
    /// Read the worker's current F5 policy after validating the actor.
    F5PolicyStatus,
    /// Check one opt-in capability without consuming a Schema 9 chat slot.
    CheckF5Capability(F5SocialCapability),
}

/// Extended F5 capability gates. IDs are trusted service-resolved targets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum F5SocialCapability {
    Whisper(i64),
    GuildChat,
    MegaphoneGlobal,
    MegaphoneChannel,
    NoteInbox(i64),
    Couple(i64),
    Plaza(F5PlazaZone),
}

/// Result of the policy and Schema 9 moderation gate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum F5SocialGate {
    Allowed,
    Disabled,
    Muted,
    Blocked,
}

/// Social projections and decisions. Contains no session, login credentials,
/// SQL error details, or chat contents.
#[derive(Debug, PartialEq, Eq)]
pub enum SocialReply {
    Changed(bool),
    Friends(Vec<i64>),
    Blocks(Vec<i64>),
    FriendRequests(Vec<FriendRequest>),
    MuteStatus(Option<SocialMute>),
    ChatPermission(ChatPermission),
    CanReceivePublicChat(bool),
    Guild(Guild),
    GuildMembers(Vec<GuildMember>),
    PlayerGuild(Option<(Guild, Vec<GuildMember>)>),
    LeadershipTransferred,
    Ranking(Vec<RankingEntry>),
    F5PolicyStatus(F5SocialPolicy),
    F5Eligibility(F5SocialGate),
}

pub(super) enum SocialRequest {
    Player {
        player: i64,
        token: Option<Token>,
        action: SocialAction,
        policy: F5SocialPolicy,
        reply: oneshot::Sender<Result<SocialReply>>,
    },
    Mute {
        player: i64,
        duration: u32,
        reason: ChatMuteReason,
        reply: oneshot::Sender<Result<i64>>,
    },
}

type F5PolicySlot = Arc<RwLock<F5SocialPolicy>>;
struct F5PolicyEntry {
    gate: Weak<Mutex<super::Admission>>,
    policy: F5PolicySlot,
}
static F5_POLICY_SLOTS: OnceLock<Mutex<HashMap<usize, F5PolicyEntry>>> = OnceLock::new();

fn f5_policy_slot(client: &Client) -> F5PolicySlot {
    let slots = F5_POLICY_SLOTS.get_or_init(|| Mutex::new(HashMap::new()));
    let key = Arc::as_ptr(&client.gate) as usize;
    let mut slots = slots
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let replace = slots.get(&key).is_none_or(|entry| {
        entry
            .gate
            .upgrade()
            .is_none_or(|gate| !Arc::ptr_eq(&gate, &client.gate))
    });
    if replace {
        slots.insert(
            key,
            F5PolicyEntry {
                gate: Arc::downgrade(&client.gate),
                policy: Arc::new(RwLock::new(F5SocialPolicy::default())),
            },
        );
    }
    slots
        .get(&key)
        .expect("policy entry inserted")
        .policy
        .clone()
}

fn f5_policy_snapshot(client: &Client) -> F5SocialPolicy {
    f5_policy_slot(client)
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone()
}

fn validate_f5_policy(policy: &F5SocialPolicy) -> Result<()> {
    ChocoChatRouter::with_f5_policy(policy.clone())
        .map(|_| ())
        .map_err(|_| Error::InvalidInput)
}

impl Client {
    /// Trusted caller API, like the economy facade: player must already be
    /// bound to its authority by the caller. Full queues return Busy.
    pub fn try_social(&self, player: i64, action: SocialAction) -> Result<Pending<SocialReply>> {
        self.submit_player_social(player, None, action)
    }

    /// Attach a validated local F5 policy to this worker's bounded social
    /// facade. Cloned clients for the same worker observe the same policy.
    pub fn set_f5_social_policy(&self, policy: F5SocialPolicy) -> Result<()> {
        validate_f5_policy(&policy)?;
        *f5_policy_slot(self)
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = policy;
        Ok(())
    }

    pub fn try_set_f5_social_policy(&self, policy: F5SocialPolicy) -> Result<()> {
        self.set_f5_social_policy(policy)
    }

    /// Alias for callers that describe policy installation as attachment.
    pub fn attach_f5_social_policy(&self, policy: F5SocialPolicy) -> Result<()> {
        self.set_f5_social_policy(policy)
    }

    pub fn try_attach_f5_social_policy(&self, policy: F5SocialPolicy) -> Result<()> {
        self.set_f5_social_policy(policy)
    }

    /// Immediate read of the configured policy. Use
    /// [`Self::try_f5_social_policy_status`] when actor validation and queue
    /// backpressure are required as part of the read.
    pub fn f5_social_policy(&self) -> F5SocialPolicy {
        f5_policy_snapshot(self)
    }

    pub fn try_f5_social_policy_status(&self, player: i64) -> Result<Pending<SocialReply>> {
        self.try_social(player, SocialAction::F5PolicyStatus)
    }

    pub fn try_f5_social_eligibility(
        &self,
        player: i64,
        capability: F5SocialCapability,
    ) -> Result<Pending<SocialReply>> {
        self.try_social(player, SocialAction::CheckF5Capability(capability))
    }

    pub fn try_f5_whisper_eligibility(
        &self,
        player: i64,
        target: i64,
    ) -> Result<Pending<SocialReply>> {
        self.try_f5_social_eligibility(player, F5SocialCapability::Whisper(target))
    }

    pub fn try_f5_guild_chat_eligibility(&self, player: i64) -> Result<Pending<SocialReply>> {
        self.try_f5_social_eligibility(player, F5SocialCapability::GuildChat)
    }

    pub fn try_f5_megaphone_global_eligibility(&self, player: i64) -> Result<Pending<SocialReply>> {
        self.try_f5_social_eligibility(player, F5SocialCapability::MegaphoneGlobal)
    }

    pub fn try_f5_megaphone_channel_eligibility(
        &self,
        player: i64,
    ) -> Result<Pending<SocialReply>> {
        self.try_f5_social_eligibility(player, F5SocialCapability::MegaphoneChannel)
    }

    pub fn try_f5_note_inbox_eligibility(
        &self,
        player: i64,
        target: i64,
    ) -> Result<Pending<SocialReply>> {
        self.try_f5_social_eligibility(player, F5SocialCapability::NoteInbox(target))
    }

    pub fn try_f5_couple_eligibility(
        &self,
        player: i64,
        target: i64,
    ) -> Result<Pending<SocialReply>> {
        self.try_f5_social_eligibility(player, F5SocialCapability::Couple(target))
    }

    pub fn try_f5_plaza_eligibility(
        &self,
        player: i64,
        zone: F5PlazaZone,
    ) -> Result<Pending<SocialReply>> {
        self.try_f5_social_eligibility(player, F5SocialCapability::Plaza(zone))
    }

    /// The bridge supplies its authenticated actor and token. The worker
    /// rechecks the session at execution time without renewing idle TTL.
    pub(crate) fn try_session_social(
        &self,
        player: i64,
        token: Token,
        action: SocialAction,
    ) -> Result<Pending<SocialReply>> {
        self.submit_player_social(player, Some(token), action)
    }

    fn submit_player_social(
        &self,
        player: i64,
        token: Option<Token>,
        action: SocialAction,
    ) -> Result<Pending<SocialReply>> {
        // Mirror the Store's raw length bound before retaining a queue entry.
        if player < 1 || matches!(&action, SocialAction::CreateGuild(name) if name.len() > 64) {
            return Err(Error::InvalidInput);
        }
        let (reply, rx) = oneshot::channel();
        self.submit(Command::Social {
            request: SocialRequest::Player {
                player,
                token,
                action,
                policy: f5_policy_snapshot(self),
                reply,
            },
            deadline: Instant::now() + self.queue_ttl,
        })?;
        Ok(Pending {
            reply: Some(rx),
            handoff: None,
        })
    }

    /// Trusted moderator API. Never available as a player action on the bridge.
    pub fn try_mute_social_player(
        &self,
        player: i64,
        duration: u32,
        reason: ChatMuteReason,
    ) -> Result<Pending<i64>> {
        if player < 1 || !(1..=accounts::SOCIAL_MUTE_MAX_SECONDS).contains(&duration) {
            return Err(Error::InvalidInput);
        }
        let (reply, rx) = oneshot::channel();
        self.submit(Command::Social {
            request: SocialRequest::Mute {
                player,
                duration,
                reason,
                reply,
            },
            deadline: Instant::now() + self.queue_ttl,
        })?;
        Ok(Pending {
            reply: Some(rx),
            handoff: None,
        })
    }
}

impl AccountService {
    pub(crate) async fn social_mute_status(&self, player: i64) -> Result<Option<SocialMute>> {
        match self
            .client
            .try_social(player, SocialAction::MuteStatus)?
            .wait()
            .await?
        {
            SocialReply::MuteStatus(mute) => Ok(mute),
            _ => Err(Error::WorkerFault),
        }
    }

    /// Minimal web-facing social facade. The caller supplies an identity that
    /// was already bound by the web session; all work remains on the account
    /// worker and never touches O1 session cosmetics.
    pub(crate) async fn social_friends(&self, player: i64) -> Result<Vec<i64>> {
        match self
            .client
            .try_social(player, SocialAction::ListFriends)?
            .wait()
            .await?
        {
            SocialReply::Friends(rows) => Ok(rows),
            _ => Err(Error::WorkerFault),
        }
    }
    pub(crate) async fn social_blocks(&self, player: i64) -> Result<Vec<i64>> {
        match self
            .client
            .try_social(player, SocialAction::ListBlocks)?
            .wait()
            .await?
        {
            SocialReply::Blocks(rows) => Ok(rows),
            _ => Err(Error::WorkerFault),
        }
    }
    pub(crate) async fn social_friend_add(&self, player: i64, target: i64) -> Result<bool> {
        match self
            .client
            .try_social(player, SocialAction::SendFriendRequest(target))?
            .wait()
            .await?
        {
            SocialReply::Changed(changed) => Ok(changed),
            _ => Err(Error::WorkerFault),
        }
    }
    pub(crate) async fn social_friend_remove(&self, player: i64, target: i64) -> Result<bool> {
        match self
            .client
            .try_social(player, SocialAction::RemoveFriend(target))?
            .wait()
            .await?
        {
            SocialReply::Changed(changed) => Ok(changed),
            _ => Err(Error::WorkerFault),
        }
    }
    pub(crate) async fn social_friend_requests(&self, player: i64) -> Result<Vec<FriendRequest>> {
        match self
            .client
            .try_social(player, SocialAction::ListFriendRequests)?
            .wait()
            .await?
        {
            SocialReply::FriendRequests(rows) => Ok(rows),
            _ => Err(Error::WorkerFault),
        }
    }
    pub(crate) async fn social_friend_respond(
        &self,
        player: i64,
        requester: i64,
        accept: bool,
    ) -> Result<bool> {
        match self
            .client
            .try_social(
                player,
                SocialAction::RespondFriendRequest { requester, accept },
            )?
            .wait()
            .await?
        {
            SocialReply::Changed(changed) => Ok(changed),
            _ => Err(Error::WorkerFault),
        }
    }
    pub(crate) async fn social_block_add(&self, player: i64, target: i64) -> Result<bool> {
        match self
            .client
            .try_social(player, SocialAction::BlockPlayer(target))?
            .wait()
            .await?
        {
            SocialReply::Changed(changed) => Ok(changed),
            _ => Err(Error::WorkerFault),
        }
    }
    pub(crate) async fn social_block_remove(&self, player: i64, target: i64) -> Result<bool> {
        match self
            .client
            .try_social(player, SocialAction::UnblockPlayer(target))?
            .wait()
            .await?
        {
            SocialReply::Changed(changed) => Ok(changed),
            _ => Err(Error::WorkerFault),
        }
    }
    pub(crate) async fn social_ranking(
        &self,
        player: i64,
        offset: u32,
        limit: u32,
    ) -> Result<Vec<accounts::RankingEntry>> {
        match self
            .client
            .try_social(player, SocialAction::Ranking { offset, limit })?
            .wait()
            .await?
        {
            SocialReply::Ranking(rows) => Ok(rows),
            _ => Err(Error::WorkerFault),
        }
    }
    pub(crate) async fn social_guild(
        &self,
        player: i64,
    ) -> Result<Option<(Guild, Vec<GuildMember>)>> {
        match self
            .client
            .try_social(player, SocialAction::GuildForPlayer)?
            .wait()
            .await?
        {
            SocialReply::PlayerGuild(guild) => Ok(guild),
            _ => Err(Error::WorkerFault),
        }
    }

    pub(crate) async fn social_guild_create(&self, player: i64, name: String) -> Result<Guild> {
        match self
            .client
            .try_social(player, SocialAction::CreateGuild(name))?
            .wait()
            .await?
        {
            SocialReply::Guild(guild) => Ok(guild),
            _ => Err(Error::WorkerFault),
        }
    }

    pub(crate) async fn social_guild_leave(&self, player: i64, guild_id: i64) -> Result<bool> {
        match self
            .client
            .try_social(player, SocialAction::LeaveGuildSelfService(guild_id))?
            .wait()
            .await?
        {
            SocialReply::Changed(changed) => Ok(changed),
            _ => Err(Error::WorkerFault),
        }
    }

    pub(crate) async fn social_guild_disband(&self, player: i64, guild_id: i64) -> Result<bool> {
        match self
            .client
            .try_social(player, SocialAction::DisbandGuild(guild_id))?
            .wait()
            .await?
        {
            SocialReply::Changed(changed) => Ok(changed),
            _ => Err(Error::WorkerFault),
        }
    }

    pub(crate) async fn social_guild_manage(
        &self,
        player: i64,
        expected_guild: Option<i64>,
        action: GuildLeaderAction,
    ) -> Result<bool> {
        match self
            .client
            .try_social(
                player,
                SocialAction::ManageGuild {
                    expected_guild,
                    action,
                },
            )?
            .wait()
            .await?
        {
            SocialReply::Changed(changed) => Ok(changed),
            _ => Err(Error::WorkerFault),
        }
    }
}

impl SocialRequest {
    pub(super) fn is_closed(&self) -> bool {
        match self {
            Self::Player { reply, .. } => reply.is_closed(),
            Self::Mute { reply, .. } => reply.is_closed(),
        }
    }

    pub(super) fn execute(self, store: &mut Store, now: Result<i64>) {
        match self {
            Self::Player {
                player,
                token,
                action,
                policy,
                reply,
            } => {
                let result = now.and_then(|now| {
                    execute_player(store, player, token, action, policy, now)
                        .map_err(Error::Backend)
                });
                let _ = reply.send(result);
            }
            Self::Mute {
                player,
                duration,
                reason,
                reply,
            } => {
                let result = now.and_then(|now| {
                    store
                        .mute_social_player(player, duration, reason, now)
                        .map_err(Error::Backend)
                });
                let _ = reply.send(result);
            }
        }
    }
}

fn execute_player(
    store: &mut Store,
    player: i64,
    token: Option<Token>,
    action: SocialAction,
    policy: F5SocialPolicy,
    now: i64,
) -> accounts::Result<SocialReply> {
    if let Some(token) = token {
        store.inspect_session(&token, player, now)?;
    }
    use SocialReply::*;
    Ok(match action {
        SocialAction::SendFriendRequest(target) => {
            Changed(store.send_friend_request(player, target, now)?)
        }
        SocialAction::AcceptFriendRequest(target) => {
            Changed(store.accept_friend_request(player, target, now)?)
        }
        SocialAction::DeclineFriendRequest(target) => {
            Changed(store.decline_friend_request(player, target, now)?)
        }
        SocialAction::RespondFriendRequest { requester, accept } => {
            Changed(store.respond_friend_request(player, requester, accept, now)?)
        }
        SocialAction::CancelFriendRequest(target) => {
            Changed(store.cancel_friend_request(player, target, now)?)
        }
        SocialAction::RemoveFriend(target) => Changed(store.remove_friend(player, target, now)?),
        SocialAction::BlockPlayer(target) => Changed(store.block_player(player, target, now)?),
        SocialAction::UnblockPlayer(target) => Changed(store.unblock_player(player, target, now)?),
        SocialAction::ListFriends => Friends(store.list_friends(player, now)?),
        SocialAction::ListBlocks => Blocks(store.list_blocks(player, now)?),
        SocialAction::ListFriendRequests => {
            FriendRequests(store.list_friend_requests(player, now)?)
        }
        SocialAction::MuteStatus => MuteStatus(store.social_mute_status(player, now)?),
        SocialAction::CheckChatPermission {
            recipient,
            message_bytes,
        } => ChatPermission(store.check_chat_permission(player, recipient, message_bytes, now)?),
        SocialAction::CanReceivePublicChat(recipient) => {
            CanReceivePublicChat(store.can_receive_public_chat(player, recipient, now)?)
        }
        SocialAction::CreateGuild(name) => Guild(store.create_guild(player, &name, now)?),
        SocialAction::JoinGuild(guild) => Changed(store.join_guild(player, guild, now)?),
        SocialAction::LeaveGuild(guild) => Changed(store.leave_guild(player, guild, now)?),
        SocialAction::LeaveGuildSelfService(guild) => {
            Changed(store.leave_guild_self_service(player, guild, now)?)
        }
        SocialAction::DisbandGuild(guild) => Changed(store.disband_guild(player, guild, now)?),
        SocialAction::SetGuildOfficer {
            guild_id,
            player: member,
            officer,
        } => Changed(store.set_guild_officer(player, guild_id, member, officer, now)?),
        SocialAction::TransferGuildLeadership {
            guild_id,
            successor,
        } => {
            store.transfer_guild_leadership(player, guild_id, successor, now)?;
            LeadershipTransferred
        }
        SocialAction::ManageGuild {
            expected_guild,
            action,
        } => {
            store.manage_guild(player, expected_guild, action, now)?;
            Changed(true)
        }
        SocialAction::ListGuildMembers(guild) => {
            GuildMembers(store.list_guild_members(guild, now)?)
        }
        SocialAction::GuildForPlayer => PlayerGuild(store.guild_for_player(player, now)?),
        SocialAction::Ranking { offset, limit } => Ranking(store.ranking(offset, limit)?),
        SocialAction::F5PolicyStatus => {
            store.social_mute_status(player, now)?;
            F5PolicyStatus(policy)
        }
        SocialAction::CheckF5Capability(capability) => {
            F5Eligibility(check_f5_capability(store, player, policy, capability, now)?)
        }
    })
}

fn check_f5_capability(
    store: &mut Store,
    player: i64,
    policy: F5SocialPolicy,
    capability: F5SocialCapability,
    now: i64,
) -> accounts::Result<F5SocialGate> {
    let enabled = match capability {
        F5SocialCapability::Whisper(_) => policy.whisper,
        F5SocialCapability::GuildChat => policy.guild_chat,
        F5SocialCapability::MegaphoneGlobal => policy.megaphone.is_some_and(|p| p.global),
        F5SocialCapability::MegaphoneChannel => policy.megaphone.is_some_and(|p| p.channel),
        F5SocialCapability::NoteInbox(_) => policy.note_inbox_cap.is_some(),
        F5SocialCapability::Couple(_) => policy.couple,
        F5SocialCapability::Plaza(zone) => policy.plaza_zones.contains_key(&zone),
    };
    if !enabled {
        return Ok(F5SocialGate::Disabled);
    }

    if store.social_mute_status(player, now)?.is_some() {
        return Ok(F5SocialGate::Muted);
    }

    let target = match capability {
        F5SocialCapability::Whisper(target)
        | F5SocialCapability::NoteInbox(target)
        | F5SocialCapability::Couple(target) => Some(target),
        F5SocialCapability::GuildChat
        | F5SocialCapability::MegaphoneGlobal
        | F5SocialCapability::MegaphoneChannel
        | F5SocialCapability::Plaza(_) => None,
    };
    if let Some(target) = target
        && !store.can_receive_public_chat(player, target, now)?
    {
        return Ok(F5SocialGate::Blocked);
    }
    Ok(F5SocialGate::Allowed)
}
