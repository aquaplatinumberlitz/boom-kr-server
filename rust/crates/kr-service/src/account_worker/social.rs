//! Schema 9 service seam. Synchronous Store calls run only on the normal
//! `kr-account-db` worker. Callers use nonblocking admission and await a reply.
//! No KR wire adapter; no message body enters the queue.

use super::{Client, Command, Error, Pending, Result};
use crate::accounts::{
    self, ChatMuteReason, ChatPermission, FriendRequest, Guild, GuildMember, RankingEntry, Store,
    Token,
};
use std::time::Instant;
use tokio::sync::oneshot;

#[cfg(test)]
mod tests;

/// Actions scoped to one actor, selected by trusted service code. The IDs in
/// these variants are targets, never the identity of the calling player.
pub enum SocialAction {
    SendFriendRequest(i64),
    AcceptFriendRequest(i64),
    DeclineFriendRequest(i64),
    CancelFriendRequest(i64),
    RemoveFriend(i64),
    BlockPlayer(i64),
    UnblockPlayer(i64),
    ListFriends,
    ListBlocks,
    ListFriendRequests,
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
    ListGuildMembers(i64),
    /// Stable ordering on unchanged data; requests do not share a snapshot.
    Ranking {
        offset: u32,
        limit: u32,
    },
}

/// Social projections and decisions. Contains no session, login credentials,
/// SQL error details, or chat contents.
#[derive(Debug, PartialEq, Eq)]
pub enum SocialReply {
    Changed(bool),
    Friends(Vec<i64>),
    Blocks(Vec<i64>),
    FriendRequests(Vec<FriendRequest>),
    ChatPermission(ChatPermission),
    CanReceivePublicChat(bool),
    Guild(Guild),
    GuildMembers(Vec<GuildMember>),
    LeadershipTransferred,
    Ranking(Vec<RankingEntry>),
}

pub(super) enum SocialRequest {
    Player {
        player: i64,
        token: Option<Token>,
        action: SocialAction,
        reply: oneshot::Sender<Result<SocialReply>>,
    },
    Mute {
        player: i64,
        duration: u32,
        reason: ChatMuteReason,
        reply: oneshot::Sender<Result<i64>>,
    },
}

impl Client {
    /// Trusted caller API, like the economy facade: player must already be
    /// bound to its authority by the caller. Full queues return Busy.
    pub fn try_social(&self, player: i64, action: SocialAction) -> Result<Pending<SocialReply>> {
        self.submit_player_social(player, None, action)
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
                reply,
            } => {
                let result = now.and_then(|now| {
                    execute_player(store, player, token, action, now).map_err(Error::Backend)
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
        SocialAction::ListGuildMembers(guild) => {
            GuildMembers(store.list_guild_members(guild, now)?)
        }
        SocialAction::Ranking { offset, limit } => Ranking(store.ranking(offset, limit)?),
    })
}
