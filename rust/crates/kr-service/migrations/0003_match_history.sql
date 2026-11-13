-- Company backend design: durable normal-match history, zero rewards only.
CREATE TABLE matches (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  boot_epoch INTEGER NOT NULL CHECK(boot_epoch>0),
  room_session INTEGER NOT NULL CHECK(room_session BETWEEN 1 AND 65535),
  room_id INTEGER NOT NULL CHECK(room_id BETWEEN 1 AND 4294967295),
  room_generation INTEGER NOT NULL CHECK(room_generation>0),
  round INTEGER NOT NULL CHECK(round>0),
  map_code INTEGER NOT NULL CHECK(map_code BETWEEN 0 AND 65535),
  map_name TEXT NOT NULL CHECK(length(map_name) BETWEEN 1 AND 64),
  started_at INTEGER NOT NULL CHECK(started_at>=0),
  ended_at INTEGER CHECK(ended_at IS NULL OR ended_at>=started_at),
  state TEXT NOT NULL CHECK(state IN('running','complete','cancelled')),
  reason TEXT,
  start_json TEXT NOT NULL CHECK(length(start_json)<=8192),
  finish_json TEXT CHECK(finish_json IS NULL OR length(finish_json)<=4096),
  CHECK((state='running')=(finish_json IS NULL)),
  CHECK((state='running')=(ended_at IS NULL)),
  CHECK((state='running')=(reason IS NULL)),
  UNIQUE(boot_epoch,room_session,room_id,room_generation,round)
) STRICT;
CREATE TABLE match_players (
  match_id INTEGER NOT NULL REFERENCES matches(id),
  account_id INTEGER NOT NULL REFERENCES accounts(id),
  player_id INTEGER NOT NULL REFERENCES players(id),
  slot INTEGER NOT NULL CHECK(slot BETWEEN 0 AND 7),
  character_key INTEGER NOT NULL CHECK(character_key BETWEEN 0 AND 8191),
  team INTEGER NOT NULL CHECK(team BETWEEN 0 AND 31),
  outcome TEXT CHECK(outcome IS NULL OR outcome IN('win','loss','draw','cancelled')),
  exit_kind TEXT CHECK(exit_kind IS NULL OR exit_kind IN('present','left','disconnected','unknown')),
  reward INTEGER NOT NULL DEFAULT 0 CHECK(reward=0),
  PRIMARY KEY(match_id,slot),
  UNIQUE(match_id,player_id),
  UNIQUE(match_id,account_id),
  CHECK((outcome IS NULL)=(exit_kind IS NULL))
) STRICT;
CREATE INDEX match_history_account ON match_players(account_id,match_id);
CREATE INDEX match_history_room ON matches(room_id,id);
CREATE TABLE player_preferences (
  player_id INTEGER PRIMARY KEY REFERENCES players(id),
  character_key INTEGER NOT NULL CHECK(character_key BETWEEN 0 AND 8191),
  revision INTEGER NOT NULL CHECK(revision>0)
) STRICT;
