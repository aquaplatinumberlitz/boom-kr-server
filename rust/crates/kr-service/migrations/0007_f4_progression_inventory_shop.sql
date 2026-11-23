-- Company design (F4): configurable progression, Lucci ledger, inventory and equip.
-- M6 match_players.reward stays 0; F4 rewards are recorded only in these tables.
-- Profiles are created lazily, so players upgraded from schema 6 need no backfill.
CREATE TABLE player_profiles (
  player_id INTEGER PRIMARY KEY REFERENCES players(id),
  exp INTEGER NOT NULL DEFAULT 0 CHECK(exp>=0),
  level INTEGER NOT NULL DEFAULT 1 CHECK(level>=1),
  lucci INTEGER NOT NULL DEFAULT 0 CHECK(lucci>=0),
  version INTEGER NOT NULL DEFAULT 1 CHECK(version>=1)
) STRICT;
CREATE TABLE ledger_entries (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  player_id INTEGER NOT NULL REFERENCES players(id),
  kind TEXT NOT NULL CHECK(kind IN('match_reward','shop_purchase','admin_grant')),
  idempotency_key TEXT NOT NULL CHECK(length(idempotency_key) BETWEEN 1 AND 128),
  request_hash TEXT NOT NULL
    CHECK(length(request_hash)=64 AND request_hash NOT GLOB '*[^0-9a-f]*'),
  exp_delta INTEGER NOT NULL,
  lucci_delta INTEGER NOT NULL,
  match_id INTEGER REFERENCES matches(id),
  sku_id TEXT CHECK(sku_id IS NULL OR length(sku_id) BETWEEN 1 AND 64),
  created_at INTEGER NOT NULL CHECK(created_at>=0),
  CHECK((kind='match_reward')=(match_id IS NOT NULL)),
  CHECK(kind<>'match_reward' OR (sku_id IS NULL AND exp_delta>=0 AND lucci_delta>=0)),
  CHECK(kind<>'shop_purchase' OR (sku_id IS NOT NULL AND exp_delta=0 AND lucci_delta<=0)),
  UNIQUE(player_id,kind,idempotency_key)
) STRICT;
CREATE INDEX ledger_player_history ON ledger_entries(player_id,id);
CREATE INDEX ledger_match ON ledger_entries(match_id) WHERE match_id IS NOT NULL;
CREATE TRIGGER ledger_entries_append_only_update BEFORE UPDATE ON ledger_entries
BEGIN SELECT RAISE(ABORT,'ledger is append-only'); END;
CREATE TRIGGER ledger_entries_append_only_delete BEFORE DELETE ON ledger_entries
BEGIN SELECT RAISE(ABORT,'ledger is append-only'); END;
CREATE TABLE player_inventory (
  player_id INTEGER NOT NULL REFERENCES players(id),
  sku_id TEXT NOT NULL CHECK(length(sku_id) BETWEEN 1 AND 64),
  grant_kind TEXT NOT NULL CHECK(grant_kind IN('permanent','stackable','timed')),
  quantity INTEGER NOT NULL CHECK(quantity>=1),
  expires_at INTEGER CHECK(expires_at IS NULL OR expires_at>=0),
  acquired_at INTEGER NOT NULL CHECK(acquired_at>=0),
  updated_at INTEGER NOT NULL CHECK(updated_at>=acquired_at),
  CHECK((grant_kind='timed')=(expires_at IS NOT NULL)),
  CHECK(grant_kind<>'permanent' OR quantity=1),
  PRIMARY KEY(player_id,sku_id)
) STRICT;
CREATE TABLE player_equipped (
  player_id INTEGER NOT NULL,
  slot TEXT NOT NULL CHECK(length(slot) BETWEEN 1 AND 32),
  sku_id TEXT NOT NULL,
  equipped_at INTEGER NOT NULL CHECK(equipped_at>=0),
  PRIMARY KEY(player_id,slot),
  FOREIGN KEY(player_id,sku_id) REFERENCES player_inventory(player_id,sku_id)
) STRICT;
CREATE INDEX player_equipped_item ON player_equipped(player_id,sku_id);
