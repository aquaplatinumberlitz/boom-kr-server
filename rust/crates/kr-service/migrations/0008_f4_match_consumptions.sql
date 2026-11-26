-- F4 receipts survive depletion of the corresponding inventory stack.
CREATE TABLE match_item_consumptions (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  player_id INTEGER NOT NULL,
  match_id INTEGER NOT NULL,
  sku_id TEXT NOT NULL CHECK(length(sku_id) BETWEEN 1 AND 64),
  idempotency_key TEXT NOT NULL CHECK(length(idempotency_key) BETWEEN 1 AND 128),
  request_hash TEXT NOT NULL
    CHECK(length(request_hash)=64 AND request_hash NOT GLOB '*[^0-9a-f]*'),
  consumed_qty INTEGER NOT NULL CHECK(consumed_qty BETWEEN 1 AND 4294967295),
  remaining_qty INTEGER NOT NULL CHECK(remaining_qty>=0),
  created_at INTEGER NOT NULL CHECK(created_at>=0),
  UNIQUE(player_id,idempotency_key),
  FOREIGN KEY(match_id,player_id) REFERENCES match_players(match_id,player_id)
) STRICT;
CREATE INDEX match_item_consumptions_match ON match_item_consumptions(match_id,player_id);
CREATE TRIGGER match_item_consumptions_append_only_update BEFORE UPDATE ON match_item_consumptions
BEGIN SELECT RAISE(ABORT,'match item consumption is append-only'); END;
CREATE TRIGGER match_item_consumptions_append_only_delete BEFORE DELETE ON match_item_consumptions
BEGIN SELECT RAISE(ABORT,'match item consumption is append-only'); END;
