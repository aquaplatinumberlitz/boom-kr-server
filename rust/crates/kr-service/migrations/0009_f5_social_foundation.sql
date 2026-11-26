-- Company design (F5-0). IDs are internal player IDs; times are UTC seconds.
CREATE TABLE friend_requests (
  sender_id INTEGER NOT NULL REFERENCES players(id),
  recipient_id INTEGER NOT NULL REFERENCES players(id),
  created_at INTEGER NOT NULL CHECK(created_at>=0),
  CHECK(sender_id<>recipient_id),
  PRIMARY KEY(sender_id,recipient_id)
) STRICT;
CREATE UNIQUE INDEX friend_request_pair
  ON friend_requests(min(sender_id,recipient_id),max(sender_id,recipient_id));
CREATE INDEX friend_requests_recipient ON friend_requests(recipient_id,sender_id);
CREATE TABLE friendships (
  player_low_id INTEGER NOT NULL REFERENCES players(id),
  player_high_id INTEGER NOT NULL REFERENCES players(id),
  created_at INTEGER NOT NULL CHECK(created_at>=0),
  CHECK(player_low_id<player_high_id),
  PRIMARY KEY(player_low_id,player_high_id)
) STRICT;
CREATE INDEX friendships_high ON friendships(player_high_id,player_low_id);
CREATE TABLE player_blocks (
  blocker_id INTEGER NOT NULL REFERENCES players(id),
  blocked_id INTEGER NOT NULL REFERENCES players(id),
  created_at INTEGER NOT NULL CHECK(created_at>=0),
  CHECK(blocker_id<>blocked_id),
  PRIMARY KEY(blocker_id,blocked_id)
) STRICT;

CREATE TABLE social_mutes (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  player_id INTEGER NOT NULL REFERENCES players(id),
  reason TEXT NOT NULL CHECK(reason IN('spam','harassment','moderation')),
  created_at INTEGER NOT NULL CHECK(created_at>=0),
  expires_at INTEGER NOT NULL CHECK(expires_at>created_at)
) STRICT;
CREATE INDEX social_mutes_active ON social_mutes(player_id,expires_at);
CREATE TABLE chat_rate_windows (
  player_id INTEGER PRIMARY KEY REFERENCES players(id),
  window_start INTEGER NOT NULL CHECK(window_start>=0),
  accepted INTEGER NOT NULL CHECK(accepted>=1)
) STRICT;
-- Store only moderation metadata. Message bodies are never written here.
CREATE TABLE chat_abuse_events (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  sender_id INTEGER NOT NULL REFERENCES players(id),
  recipient_id INTEGER REFERENCES players(id),
  reason TEXT NOT NULL CHECK(reason IN('blocked','muted','invalid_length','rate_limited')),
  message_bytes INTEGER NOT NULL CHECK(message_bytes>=0),
  accepted_in_window INTEGER NOT NULL CHECK(accepted_in_window>=0),
  at_utc INTEGER NOT NULL CHECK(at_utc>=0),
  CHECK(recipient_id IS NULL OR sender_id<>recipient_id)
) STRICT;
CREATE INDEX chat_abuse_sender_history ON chat_abuse_events(sender_id,id);

-- The deferred FK and the partial unique index together require exactly one
-- leader for each surviving guild at commit, including during ownership transfer.
CREATE TABLE guilds (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL CHECK(length(name) BETWEEN 3 AND 24
    AND name NOT GLOB '*[^a-zA-Z0-9 ]*' AND name=trim(name) AND instr(name,'  ')=0),
  name_key TEXT NOT NULL UNIQUE CHECK(name_key=lower(name)),
  leader_player_id INTEGER NOT NULL,
  leader_role TEXT GENERATED ALWAYS AS ('leader') VIRTUAL,
  created_at INTEGER NOT NULL CHECK(created_at>=0),
  FOREIGN KEY(id,leader_player_id,leader_role)
    REFERENCES guild_members(guild_id,player_id,role) DEFERRABLE INITIALLY DEFERRED
) STRICT;
CREATE TABLE guild_members (
  guild_id INTEGER NOT NULL REFERENCES guilds(id) ON DELETE CASCADE,
  player_id INTEGER NOT NULL UNIQUE REFERENCES players(id),
  role TEXT NOT NULL CHECK(role IN('leader','officer','member')),
  joined_at INTEGER NOT NULL CHECK(joined_at>=0),
  PRIMARY KEY(guild_id,player_id),
  UNIQUE(guild_id,player_id,role)
) STRICT;
CREATE UNIQUE INDEX guild_single_leader ON guild_members(guild_id) WHERE role='leader';

-- Ranking reads Schema 7 profiles and completed match wins in place.
CREATE INDEX ranking_wins_by_player ON match_players(player_id,match_id) WHERE outcome='win';
