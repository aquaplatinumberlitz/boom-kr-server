//! Social storage and company policy. Player actions take server-resolved IDs;
//! local operator actions take an Admin and audit changes in the same transaction.
use super::*;

#[cfg(test)]
#[path = "social_tests.rs"]
mod tests;

pub const FRIEND_CAP: i64 = 100;
pub const BLOCK_CAP: i64 = 100;
/// Separate incoming and outgoing caps keep request lists bounded.
pub const FRIEND_REQUEST_CAP: i64 = 100;
pub const GUILD_MEMBER_CAP: i64 = 100;
pub const CHAT_MAX_BYTES: usize = 512;
pub const CHAT_WINDOW_SECONDS: i64 = 10;
pub const CHAT_WINDOW_CAP: i64 = 5;
pub const SOCIAL_MUTE_MAX_SECONDS: u32 = 30 * 86_400;
pub const RANKING_PAGE_MAX: u32 = 100;
pub const RANKING_OFFSET_MAX: u32 = 10_000;

#[derive(Debug, Serialize)]
pub struct ChatAbuseEvent {
    pub id: i64,
    pub sender_id: i64,
    pub recipient_id: Option<i64>,
    pub reason: String,
    pub message_bytes: i64,
    pub accepted_in_window: i64,
    pub at_utc: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct FriendRequest {
    pub sender_id: i64,
    pub recipient_id: i64,
    pub created_at: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChatMuteReason {
    Spam,
    Harassment,
    Moderation,
}
impl ChatMuteReason {
    fn code(self) -> &'static str {
        match self {
            Self::Spam => "spam",
            Self::Harassment => "harassment",
            Self::Moderation => "moderation",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChatPermission {
    Allowed,
    Blocked,
    Muted,
    InvalidLength,
    RateLimited,
}
impl ChatPermission {
    fn denial_code(self) -> Option<&'static str> {
        match self {
            Self::Allowed => None,
            Self::Blocked => Some("blocked"),
            Self::Muted => Some("muted"),
            Self::InvalidLength => Some("invalid_length"),
            Self::RateLimited => Some("rate_limited"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GuildRole {
    Leader,
    Officer,
    Member,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Guild {
    pub id: i64,
    pub name: String,
    pub leader_player_id: i64,
    pub created_at: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct GuildMember {
    pub player_id: i64,
    pub role: GuildRole,
    pub joined_at: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RankingEntry {
    pub player_id: i64,
    pub display_name: String,
    pub level: i64,
    pub exp: i64,
    pub wins: i64,
}

fn active_player(db: &Connection, player: i64, now: i64) -> Result<()> {
    let enabled: bool = db.query_row(
        "SELECT EXISTS(SELECT 1 FROM players p JOIN accounts a ON a.id=p.account_id
         WHERE p.id=?1 AND a.disabled=0 AND a.login NOT GLOB '~deleted_*'
         AND NOT EXISTS(SELECT 1 FROM guest_accounts g WHERE g.account_id=a.id AND g.expires_at<=?2))",
        params![player, now],
        |row| row.get(0),
    )?;
    if !enabled {
        return Err(Error::InvalidCredentials);
    }
    Ok(())
}

fn distinct_players(a: i64, b: i64) -> Result<(i64, i64)> {
    if a <= 0 || b <= 0 || a == b {
        return Err(Error::InvalidInput);
    }
    Ok((a.min(b), a.max(b)))
}

fn blocked(db: &Connection, a: i64, b: i64) -> Result<bool> {
    Ok(db.query_row(
        "SELECT EXISTS(SELECT 1 FROM player_blocks WHERE
          (blocker_id=?1 AND blocked_id=?2) OR (blocker_id=?2 AND blocked_id=?1))",
        params![a, b],
        |row| row.get(0),
    )?)
}

fn friends(db: &Connection, pair: (i64, i64)) -> Result<bool> {
    Ok(db.query_row(
        "SELECT EXISTS(SELECT 1 FROM friendships WHERE player_low_id=?1 AND player_high_id=?2)",
        params![pair.0, pair.1],
        |row| row.get(0),
    )?)
}

fn check_friend_capacity(db: &Connection, player: i64) -> Result<()> {
    let count: i64 = db.query_row(
        "SELECT count(*) FROM friendships WHERE player_low_id=?1 OR player_high_id=?1",
        [player],
        |row| row.get(0),
    )?;
    if count >= FRIEND_CAP {
        return Err(Error::Exhausted);
    }
    Ok(())
}

impl Store {
    /// Returns true when a new invitation is saved. An opposite pending
    /// invitation requires an explicit accept; sending never creates a friendship.
    pub fn send_friend_request(&mut self, sender: i64, recipient: i64, now: i64) -> Result<bool> {
        let pair = distinct_players(sender, recipient)?;
        self.at(now, |tx| {
            active_player(tx, sender, now)?;
            active_player(tx, recipient, now)?;
            if blocked(tx, sender, recipient)? {
                return Err(Error::Conflict);
            }
            if friends(tx, pair)? {
                return Ok(false);
            }
            let existing: Option<i64> = tx
                .query_row(
                    "SELECT sender_id FROM friend_requests WHERE
                     min(sender_id,recipient_id)=?1 AND max(sender_id,recipient_id)=?2",
                    params![pair.0, pair.1],
                    |row| row.get(0),
                )
                .optional()?;
            if let Some(existing) = existing {
                return if existing == sender {
                    Ok(false)
                } else {
                    Err(Error::Conflict)
                };
            }
            check_friend_capacity(tx, sender)?;
            check_friend_capacity(tx, recipient)?;
            let full: bool = tx.query_row(
                "SELECT (SELECT count(*) FROM friend_requests WHERE sender_id=?1)>=?3
                     OR (SELECT count(*) FROM friend_requests WHERE recipient_id=?2)>=?3",
                params![sender, recipient, FRIEND_REQUEST_CAP],
                |row| row.get(0),
            )?;
            if full {
                return Err(Error::Exhausted);
            }
            Ok(tx.execute(
                "INSERT INTO friend_requests VALUES(?1,?2,?3)",
                params![sender, recipient, now],
            )? == 1)
        })
    }

    /// Only the recipient can accept a pending invitation from the named sender.
    pub fn accept_friend_request(&mut self, recipient: i64, sender: i64, now: i64) -> Result<bool> {
        let pair = distinct_players(sender, recipient)?;
        self.at(now, |tx| {
            active_player(tx, recipient, now)?;
            active_player(tx, sender, now)?;
            if blocked(tx, sender, recipient)? {
                return Err(Error::Conflict);
            }
            if tx.execute(
                "DELETE FROM friend_requests WHERE sender_id=?1 AND recipient_id=?2",
                params![sender, recipient],
            )? == 0
            {
                return Ok(false);
            }
            check_friend_capacity(tx, sender)?;
            check_friend_capacity(tx, recipient)?;
            tx.execute(
                "INSERT INTO friendships VALUES(?1,?2,?3)",
                params![pair.0, pair.1, now],
            )?;
            Ok(true)
        })
    }

    pub fn decline_friend_request(
        &mut self,
        recipient: i64,
        sender: i64,
        now: i64,
    ) -> Result<bool> {
        self.remove_friend_request(recipient, sender, recipient, now)
    }

    /// Respond to a pending request with an explicit recipient ownership check.
    /// A caller that is neither the request recipient nor the requester cannot
    /// consume or alter the request.
    pub fn respond_friend_request(
        &mut self,
        recipient: i64,
        sender: i64,
        accept: bool,
        now: i64,
    ) -> Result<bool> {
        distinct_players(sender, recipient)?;
        self.at(now, |tx| {
            active_player(tx, recipient, now)?;
            active_player(tx, sender, now)?;
            let pending_recipient: Option<i64> = tx
                .query_row(
                    "SELECT recipient_id FROM friend_requests
                     WHERE sender_id=?1 AND recipient_id=?2",
                    params![sender, recipient],
                    |row| row.get(0),
                )
                .optional()?;
            if pending_recipient.is_none() {
                let has_other_recipient: bool = tx.query_row(
                    "SELECT EXISTS(SELECT 1 FROM friend_requests
                     WHERE sender_id=?1 AND recipient_id<>?2)",
                    params![sender, recipient],
                    |row| row.get(0),
                )?;
                if has_other_recipient {
                    return Err(Error::Conflict);
                }
                return Ok(false);
            }
            if blocked(tx, sender, recipient)? {
                return Err(Error::Conflict);
            }
            if tx.execute(
                "DELETE FROM friend_requests WHERE sender_id=?1 AND recipient_id=?2",
                params![sender, recipient],
            )? == 0
            {
                return Ok(false);
            }
            if accept {
                check_friend_capacity(tx, sender)?;
                check_friend_capacity(tx, recipient)?;
                tx.execute(
                    "INSERT INTO friendships VALUES(?1,?2,?3)",
                    params![sender.min(recipient), sender.max(recipient), now],
                )?;
            }
            Ok(true)
        })
    }

    pub fn cancel_friend_request(&mut self, sender: i64, recipient: i64, now: i64) -> Result<bool> {
        self.remove_friend_request(sender, sender, recipient, now)
    }

    fn remove_friend_request(
        &mut self,
        actor: i64,
        sender: i64,
        recipient: i64,
        now: i64,
    ) -> Result<bool> {
        distinct_players(sender, recipient)?;
        self.at(now, |tx| {
            active_player(tx, actor, now)?;
            Ok(tx.execute(
                "DELETE FROM friend_requests WHERE sender_id=?1 AND recipient_id=?2",
                params![sender, recipient],
            )? == 1)
        })
    }

    pub fn remove_friend(&mut self, player: i64, friend: i64, now: i64) -> Result<bool> {
        let pair = distinct_players(player, friend)?;
        self.at(now, |tx| {
            active_player(tx, player, now)?;
            Ok(tx.execute(
                "DELETE FROM friendships WHERE player_low_id=?1 AND player_high_id=?2",
                params![pair.0, pair.1],
            )? == 1)
        })
    }

    /// A block removes a friendship and either direction of pending invitation.
    /// Removing the block does not restore those relationships.
    pub fn block_player(&mut self, blocker: i64, target: i64, now: i64) -> Result<bool> {
        let pair = distinct_players(blocker, target)?;
        self.at(now, |tx| {
            active_player(tx, blocker, now)?;
            let (exists, count): (bool, i64) = tx.query_row(
                "SELECT coalesce(max(blocked_id=?2),0), count(*) FROM player_blocks WHERE blocker_id=?1",
                params![blocker, target],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )?;
            if exists {
                return Ok(false);
            }
            if count >= BLOCK_CAP {
                return Err(Error::Exhausted);
            }
            tx.execute("INSERT INTO player_blocks VALUES(?1,?2,?3)", params![blocker, target, now])?;
            tx.execute(
                "DELETE FROM friendships WHERE player_low_id=?1 AND player_high_id=?2",
                params![pair.0, pair.1],
            )?;
            tx.execute(
                "DELETE FROM friend_requests WHERE
                 min(sender_id,recipient_id)=?1 AND max(sender_id,recipient_id)=?2",
                params![pair.0, pair.1],
            )?;
            Ok(true)
        })
    }

    pub fn unblock_player(&mut self, blocker: i64, target: i64, now: i64) -> Result<bool> {
        distinct_players(blocker, target)?;
        self.at(now, |tx| {
            active_player(tx, blocker, now)?;
            Ok(tx.execute(
                "DELETE FROM player_blocks WHERE blocker_id=?1 AND blocked_id=?2",
                params![blocker, target],
            )? == 1)
        })
    }

    pub fn list_friends(&mut self, player: i64, now: i64) -> Result<Vec<i64>> {
        self.social_ids(
            player,
            now,
            "SELECT player_high_id FROM friendships WHERE player_low_id=?1
             UNION ALL SELECT player_low_id FROM friendships WHERE player_high_id=?1
             ORDER BY 1",
        )
    }

    pub fn list_blocks(&mut self, player: i64, now: i64) -> Result<Vec<i64>> {
        self.social_ids(
            player,
            now,
            "SELECT blocked_id FROM player_blocks WHERE blocker_id=?1 ORDER BY blocked_id",
        )
    }

    fn social_ids(&mut self, player: i64, now: i64, sql: &str) -> Result<Vec<i64>> {
        positive_id(player)?;
        self.at(now, |tx| {
            active_player(tx, player, now)?;
            Ok(tx
                .prepare(sql)?
                .query_map([player], |row| row.get(0))?
                .collect::<std::result::Result<_, _>>()?)
        })
    }

    pub fn list_friend_requests(&mut self, player: i64, now: i64) -> Result<Vec<FriendRequest>> {
        self.at(now, |tx| {
            active_player(tx, player, now)?;
            Ok(tx
                .prepare(
                    "SELECT sender_id,recipient_id,created_at FROM friend_requests
                 WHERE sender_id=?1 OR recipient_id=?1 ORDER BY created_at,sender_id,recipient_id",
                )?
                .query_map([player], |row| {
                    Ok(FriendRequest {
                        sender_id: row.get(0)?,
                        recipient_id: row.get(1)?,
                        created_at: row.get(2)?,
                    })
                })?
                .collect::<std::result::Result<_, _>>()?)
        })
    }

    /// Trusted moderator operation. A new grant never shortens another one.
    /// A mute expires exactly at its UTC deadline unless cleared by an operator.
    pub fn mute_social_player(
        &mut self,
        player: i64,
        duration: u32,
        reason: ChatMuteReason,
        now: i64,
    ) -> Result<i64> {
        positive_id(player)?;
        validate_mute_duration(duration)?;
        self.at(now, |tx| insert_mute(tx, player, duration, reason, now))
    }

    /// Checks and consumes one send slot atomically. Only `Ok(Allowed)` permits
    /// sending. Pass a recipient for any addressed message; for a broadcast, call
    /// `can_receive_public_chat` for each recipient before delivery.
    pub fn check_chat_permission(
        &mut self,
        sender: i64,
        recipient: Option<i64>,
        message_bytes: usize,
        now: i64,
    ) -> Result<ChatPermission> {
        if let Some(recipient) = recipient {
            distinct_players(sender, recipient)?;
        }
        self.at(now, |tx| {
            active_player(tx, sender, now)?;
            let is_blocked = if let Some(recipient) = recipient {
                active_player(tx, recipient, now)?;
                blocked(tx, sender, recipient)?
            } else {
                false
            };
            let muted: bool = tx.query_row(
                "SELECT EXISTS(SELECT 1 FROM social_mutes WHERE player_id=?1 AND expires_at>?2)",
                params![sender, now],
                |row| row.get(0),
            )?;
            let window = now - now.rem_euclid(CHAT_WINDOW_SECONDS);
            let accepted: i64 = tx.query_row(
                "SELECT coalesce((SELECT accepted FROM chat_rate_windows
                                  WHERE player_id=?1 AND window_start=?2),0)",
                params![sender, window],
                |row| row.get(0),
            )?;
            let permission = if is_blocked {
                ChatPermission::Blocked
            } else if muted {
                ChatPermission::Muted
            } else if !(1..=CHAT_MAX_BYTES).contains(&message_bytes) {
                ChatPermission::InvalidLength
            } else if accepted >= CHAT_WINDOW_CAP {
                ChatPermission::RateLimited
            } else {
                ChatPermission::Allowed
            };
            if let Some(reason) = permission.denial_code() {
                tx.execute(
                    "INSERT INTO chat_abuse_events(sender_id,recipient_id,reason,message_bytes,accepted_in_window,at_utc)
                     VALUES(?1,?2,?3,?4,?5,?6)",
                    params![sender, recipient, reason, i64::try_from(message_bytes).unwrap_or(i64::MAX), accepted, now],
                )?;
            } else {
                tx.execute(
                    "INSERT INTO chat_rate_windows VALUES(?1,?2,?3)
                     ON CONFLICT(player_id) DO UPDATE SET window_start=excluded.window_start,accepted=excluded.accepted",
                    params![sender, window, accepted + 1],
                )?;
            }
            Ok(permission)
        })
    }

    /// Delivery filter for an already authorized broadcast. It never reserves a
    /// rate slot. Only `Ok(true)` permits this recipient to receive the message.
    pub fn can_receive_public_chat(
        &mut self,
        sender: i64,
        recipient: i64,
        now: i64,
    ) -> Result<bool> {
        self.at(now, |tx| {
            active_player(tx, sender, now)?;
            active_player(tx, recipient, now)?;
            Ok(!blocked(tx, sender, recipient)?)
        })
    }

    pub fn create_guild(&mut self, leader: i64, name: &str, now: i64) -> Result<Guild> {
        let name = normalize_guild_name(name)?;
        self.at(now, |tx| insert_guild(tx, leader, name, now))
    }

    pub fn join_guild(&mut self, player: i64, guild_id: i64, now: i64) -> Result<bool> {
        self.at(now, |tx| insert_guild_member(tx, player, guild_id, now))
    }

    pub fn leave_guild(&mut self, player: i64, guild_id: i64, now: i64) -> Result<bool> {
        self.at(now, |tx| {
            active_player(tx, player, now)?;
            delete_guild_member(tx, guild_id, player, guild_owner(tx, guild_id)?)
        })
    }

    pub fn disband_guild(&mut self, leader: i64, guild_id: i64, now: i64) -> Result<bool> {
        self.at(now, |tx| {
            active_player(tx, leader, now)?;
            let Some(owner) = guild_owner(tx, guild_id)? else {
                return Ok(false);
            };
            if owner != leader {
                return Err(Error::Conflict);
            }
            Ok(tx.execute("DELETE FROM guilds WHERE id=?1", [guild_id])? == 1)
        })
    }

    pub fn set_guild_officer(
        &mut self,
        leader: i64,
        guild_id: i64,
        player: i64,
        officer: bool,
        now: i64,
    ) -> Result<bool> {
        distinct_players(leader, player)?;
        self.at(now, |tx| {
            require_guild_owner(tx, leader, guild_id, now)?;
            let role = guild_role(tx, guild_id, player)?.ok_or(Error::InvalidInput)?;
            let updated = if officer { "officer" } else { "member" };
            if role == updated {
                return Ok(false);
            }
            Ok(tx.execute(
                "UPDATE guild_members SET role=?3 WHERE guild_id=?1 AND player_id=?2",
                params![guild_id, player, updated],
            )? == 1)
        })
    }

    pub fn transfer_guild_leadership(
        &mut self,
        leader: i64,
        guild_id: i64,
        successor: i64,
        now: i64,
    ) -> Result<()> {
        distinct_players(leader, successor)?;
        self.at(now, |tx| {
            require_guild_owner(tx, leader, guild_id, now)?;
            active_player(tx, successor, now)?;
            guild_role(tx, guild_id, successor)?.ok_or(Error::InvalidInput)?;
            tx.execute(
                "UPDATE guild_members SET role='member' WHERE guild_id=?1 AND player_id=?2",
                params![guild_id, leader],
            )?;
            tx.execute(
                "UPDATE guild_members SET role='leader' WHERE guild_id=?1 AND player_id=?2",
                params![guild_id, successor],
            )?;
            tx.execute(
                "UPDATE guilds SET leader_player_id=?1 WHERE id=?2",
                params![successor, guild_id],
            )?;
            Ok(())
        })
    }

    pub fn list_guild_members(&mut self, guild_id: i64, now: i64) -> Result<Vec<GuildMember>> {
        self.at(now, |tx| guild_members(tx, guild_id))
    }

    pub fn get_guild(&mut self, guild_id: i64) -> Result<(Guild, Vec<GuildMember>)> {
        positive_id(guild_id)?;
        let tx = self.db.transaction()?;
        let guild = tx
            .query_row(
                "SELECT name,leader_player_id,created_at FROM guilds WHERE id=?1",
                [guild_id],
                |r| {
                    Ok(Guild {
                        id: guild_id,
                        name: r.get(0)?,
                        leader_player_id: r.get(1)?,
                        created_at: r.get(2)?,
                    })
                },
            )
            .optional()?
            .ok_or(Error::InvalidInput)?;
        let members = guild_members(&tx, guild_id)?;
        tx.commit()?;
        Ok((guild, members))
    }

    /// Returns the guild containing the player, including its current roster.
    /// A player without membership is represented by `None`.
    pub fn guild_for_player(
        &mut self,
        player: i64,
        now: i64,
    ) -> Result<Option<(Guild, Vec<GuildMember>)>> {
        positive_id(player)?;
        self.at(now, |tx| {
            active_player(tx, player, now)?;
            let Some(guild_id) = tx
                .query_row(
                    "SELECT guild_id FROM guild_members WHERE player_id=?1",
                    [player],
                    |row| row.get(0),
                )
                .optional()?
            else {
                return Ok(None);
            };
            let guild = tx
                .query_row(
                    "SELECT name,leader_player_id,created_at FROM guilds WHERE id=?1",
                    [guild_id],
                    |row| {
                        Ok(Guild {
                            id: guild_id,
                            name: row.get(0)?,
                            leader_player_id: row.get(1)?,
                            created_at: row.get(2)?,
                        })
                    },
                )
                .optional()?
                .ok_or(Error::Storage)?;
            Ok(Some((guild, guild_members(tx, guild_id)?)))
        })
    }

    pub fn get_player_guild(
        &mut self,
        player: i64,
        now: i64,
    ) -> Result<Option<(Guild, Vec<GuildMember>)>> {
        self.guild_for_player(player, now)
    }

    pub fn list_chat_abuse(
        &self,
        sender_id: Option<i64>,
        before_id: Option<i64>,
        limit: u8,
    ) -> Result<Vec<ChatAbuseEvent>> {
        if !(1..=100).contains(&limit) {
            return Err(Error::InvalidInput);
        }
        if let Some(id) = before_id {
            positive_id(id)?;
        }
        // Keep a separate sender predicate so SQLite can use the sender history index.
        let predicate = if let Some(sender) = sender_id {
            positive_id(sender)?;
            "sender_id=?1 AND id<=?2"
        } else {
            "id<=?2"
        };
        let last_id = before_id.map_or(i64::MAX, |id| id - 1);
        let mut query = self.db.prepare(&format!(
            "SELECT id,sender_id,recipient_id,reason,message_bytes,accepted_in_window,at_utc
             FROM chat_abuse_events WHERE {predicate} ORDER BY id DESC LIMIT ?3"
        ))?;
        Ok(query
            .query_map(params![sender_id, last_id, limit], |r| {
                Ok(ChatAbuseEvent {
                    id: r.get(0)?,
                    sender_id: r.get(1)?,
                    recipient_id: r.get(2)?,
                    reason: r.get(3)?,
                    message_bytes: r.get(4)?,
                    accepted_in_window: r.get(5)?,
                    at_utc: r.get(6)?,
                })
            })?
            .collect::<std::result::Result<_, _>>()?)
    }

    pub fn admin_set_social_mute(
        &mut self,
        admin: &Admin<'_>,
        player: i64,
        duration: u32,
        reason: ChatMuteReason,
    ) -> Result<i64> {
        validate_mute_duration(duration)?;
        self.social_admin_write(
            admin,
            "social_mute_set",
            ("player", player),
            Some(player),
            |tx, now| insert_mute(tx, player, duration, reason, now),
        )
    }

    pub fn admin_clear_social_mute(&mut self, admin: &Admin<'_>, player: i64) -> Result<usize> {
        self.social_admin_write(
            admin,
            "social_mute_clear",
            ("player", player),
            Some(player),
            |tx, now| {
                Ok(tx.execute(
                    "DELETE FROM social_mutes WHERE player_id=?1 AND expires_at>?2",
                    params![player, now],
                )?)
            },
        )
    }

    pub fn admin_create_guild(
        &mut self,
        admin: &Admin<'_>,
        leader: i64,
        name: &str,
    ) -> Result<Guild> {
        let name = normalize_guild_name(name)?;
        self.social_admin_write(
            admin,
            "social_guild_create",
            ("player", leader),
            Some(leader),
            |tx, now| insert_guild(tx, leader, name, now),
        )
    }

    pub fn admin_add_guild_member(
        &mut self,
        admin: &Admin<'_>,
        guild_id: i64,
        player: i64,
    ) -> Result<bool> {
        self.social_admin_write(
            admin,
            "social_guild_member_add",
            ("guild", guild_id),
            Some(player),
            |tx, now| insert_guild_member(tx, player, guild_id, now),
        )
    }

    pub fn admin_remove_guild_member(
        &mut self,
        admin: &Admin<'_>,
        guild_id: i64,
        player: i64,
    ) -> Result<bool> {
        self.social_admin_write(
            admin,
            "social_guild_member_remove",
            ("guild", guild_id),
            Some(player),
            |tx, _| {
                let owner = guild_owner(tx, guild_id)?.ok_or(Error::InvalidInput)?;
                delete_guild_member(tx, guild_id, player, Some(owner))
            },
        )
    }

    pub fn admin_disband_guild(&mut self, admin: &Admin<'_>, guild_id: i64) -> Result<()> {
        self.social_admin_write(
            admin,
            "social_guild_disband",
            ("guild", guild_id),
            None,
            |tx, _| {
                if tx.execute("DELETE FROM guilds WHERE id=?1", [guild_id])? != 1 {
                    return Err(Error::InvalidInput);
                }
                Ok(())
            },
        )
    }

    fn social_admin_write<T>(
        &mut self,
        admin: &Admin<'_>,
        op: &str,
        (kind, id): (&str, i64),
        player: Option<i64>,
        apply: impl FnOnce(&Transaction<'_>, i64) -> Result<T>,
    ) -> Result<T> {
        admin.validate()?;
        positive_id(id)?;
        if let Some(player) = player {
            positive_id(player)?;
        }
        let request = hex(&self.random::<32>()?);
        self.at_admin(None, |tx, now| {
            let account = player.map(|player| Self::player_exists(tx, player)).transpose()?;
            let result = apply(tx, now)?;
            tx.execute(
                "INSERT INTO admin_audit(request_id,actor,operation,account_id,reason,at_utc,details,target_kind,target_id)
                 VALUES(?1,?2,?3,?4,?5,?6,'{}',?7,?8)",
                params![request, admin.actor, op, account, admin.reason, now, kind, id.to_string()],
            )?;
            Ok(result)
        })
    }

    /// Local ranking of active registered players, including default profiles.
    /// Ordering is stable for unchanged data; separate pages share no snapshot.
    pub fn ranking(&self, offset: u32, limit: u32) -> Result<Vec<RankingEntry>> {
        if limit == 0 || limit > RANKING_PAGE_MAX || offset > RANKING_OFFSET_MAX {
            return Err(Error::InvalidInput);
        }
        let mut stmt = self.db.prepare(
            "SELECT p.id,p.display_name,coalesce(pp.level,1) AS level,coalesce(pp.exp,0) AS exp,
                    (SELECT count(*) FROM match_players mp JOIN matches m ON m.id=mp.match_id
                     WHERE mp.player_id=p.id AND mp.outcome='win' AND m.state='complete') AS wins
             FROM players p LEFT JOIN player_profiles pp ON pp.player_id=p.id
             JOIN accounts a ON a.id=p.account_id
             WHERE a.disabled=0 AND a.login NOT GLOB '~deleted_*'
             AND NOT EXISTS(SELECT 1 FROM guest_accounts g WHERE g.account_id=a.id)
             ORDER BY level DESC,exp DESC,wins DESC,p.id ASC LIMIT ?1 OFFSET ?2",
        )?;
        Ok(stmt
            .query_map(params![limit, offset], |r| {
                Ok(RankingEntry {
                    player_id: r.get(0)?,
                    display_name: r.get(1)?,
                    level: r.get(2)?,
                    exp: r.get(3)?,
                    wins: r.get(4)?,
                })
            })?
            .collect::<std::result::Result<_, _>>()?)
    }
}

fn positive_id(id: i64) -> Result<()> {
    if id <= 0 {
        return Err(Error::InvalidInput);
    }
    Ok(())
}

fn validate_mute_duration(duration: u32) -> Result<()> {
    if !(1..=SOCIAL_MUTE_MAX_SECONDS).contains(&duration) {
        return Err(Error::InvalidInput);
    }
    Ok(())
}

fn insert_mute(
    db: &Connection,
    player: i64,
    duration: u32,
    reason: ChatMuteReason,
    now: i64,
) -> Result<i64> {
    let expires = now
        .checked_add(i64::from(duration))
        .ok_or(Error::Exhausted)?;
    db.execute(
        "INSERT INTO social_mutes(player_id,reason,created_at,expires_at) VALUES(?1,?2,?3,?4)",
        params![player, reason.code(), now, expires],
    )?;
    Ok(expires)
}

fn insert_guild(db: &Connection, leader: i64, name: String, now: i64) -> Result<Guild> {
    active_player(db, leader, now)?;
    db.execute(
        "INSERT INTO guilds(name,name_key,leader_player_id,created_at) VALUES(?1,?2,?3,?4)",
        params![name, name.to_ascii_lowercase(), leader, now],
    )?;
    let id = db.last_insert_rowid();
    db.execute(
        "INSERT INTO guild_members(guild_id,player_id,role,joined_at) VALUES(?1,?2,'leader',?3)",
        params![id, leader, now],
    )?;
    Ok(Guild {
        id,
        name,
        leader_player_id: leader,
        created_at: now,
    })
}

fn insert_guild_member(db: &Connection, player: i64, guild_id: i64, now: i64) -> Result<bool> {
    active_player(db, player, now)?;
    let current: Option<i64> = db
        .query_row(
            "SELECT guild_id FROM guild_members WHERE player_id=?1",
            [player],
            |r| r.get(0),
        )
        .optional()?;
    if let Some(current) = current {
        return if current == guild_id {
            Ok(false)
        } else {
            Err(Error::Conflict)
        };
    }
    guild_owner(db, guild_id)?.ok_or(Error::InvalidInput)?;
    let members: i64 = db.query_row(
        "SELECT count(*) FROM guild_members WHERE guild_id=?1",
        [guild_id],
        |r| r.get(0),
    )?;
    if members >= GUILD_MEMBER_CAP {
        return Err(Error::Exhausted);
    }
    Ok(db.execute(
        "INSERT INTO guild_members(guild_id,player_id,role,joined_at) VALUES(?1,?2,'member',?3)",
        params![guild_id, player, now],
    )? == 1)
}

fn delete_guild_member(
    db: &Connection,
    guild_id: i64,
    player: i64,
    owner: Option<i64>,
) -> Result<bool> {
    if owner == Some(player) {
        return Err(Error::Conflict);
    }
    Ok(db.execute(
        "DELETE FROM guild_members WHERE guild_id=?1 AND player_id=?2",
        params![guild_id, player],
    )? == 1)
}

fn guild_members(db: &Connection, guild_id: i64) -> Result<Vec<GuildMember>> {
    Ok(db
        .prepare(
            "SELECT player_id,role,joined_at FROM guild_members WHERE guild_id=?1
         ORDER BY CASE role WHEN 'leader' THEN 0 WHEN 'officer' THEN 1 ELSE 2 END,player_id",
        )?
        .query_map([guild_id], |r| {
            Ok(GuildMember {
                player_id: r.get(0)?,
                role: parse_role(&r.get::<_, String>(1)?)?,
                joined_at: r.get(2)?,
            })
        })?
        .collect::<std::result::Result<_, _>>()?)
}

fn normalize_guild_name(value: &str) -> Result<String> {
    if value.len() > 64
        || !value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b' ')
    {
        return Err(Error::InvalidInput);
    }
    let normalized = value.split_ascii_whitespace().collect::<Vec<_>>().join(" ");
    if !(3..=24).contains(&normalized.len()) {
        return Err(Error::InvalidInput);
    }
    Ok(normalized)
}

fn guild_owner(db: &Connection, guild_id: i64) -> Result<Option<i64>> {
    Ok(db
        .query_row(
            "SELECT leader_player_id FROM guilds WHERE id=?1",
            [guild_id],
            |r| r.get(0),
        )
        .optional()?)
}

fn require_guild_owner(db: &Connection, player: i64, guild_id: i64, now: i64) -> Result<()> {
    active_player(db, player, now)?;
    if guild_owner(db, guild_id)? != Some(player) {
        return Err(Error::Conflict);
    }
    Ok(())
}

fn guild_role(db: &Connection, guild_id: i64, player: i64) -> Result<Option<String>> {
    Ok(db
        .query_row(
            "SELECT role FROM guild_members WHERE guild_id=?1 AND player_id=?2",
            params![guild_id, player],
            |r| r.get(0),
        )
        .optional()?)
}

fn parse_role(role: &str) -> rusqlite::Result<GuildRole> {
    match role {
        "leader" => Ok(GuildRole::Leader),
        "officer" => Ok(GuildRole::Officer),
        "member" => Ok(GuildRole::Member),
        _ => Err(rusqlite::Error::InvalidQuery),
    }
}
