-- Company backend design. No client KR wire fields are inferred here.
CREATE TABLE schema_migrations (
  version INTEGER PRIMARY KEY,
  sha256 TEXT NOT NULL CHECK(length(sha256)=64)
) STRICT;
CREATE TABLE runtime_state (
  id INTEGER PRIMARY KEY CHECK(id=1),
  epoch INTEGER NOT NULL CHECK(epoch>=0),
  last_utc INTEGER NOT NULL CHECK(last_utc>=0),
  next_wire_id INTEGER NOT NULL CHECK(next_wire_id BETWEEN 1 AND 4294967296)
) STRICT;
INSERT INTO runtime_state VALUES(1,0,0,1);
CREATE TABLE accounts (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  login TEXT NOT NULL UNIQUE CHECK(length(login) BETWEEN 3 AND 32),
  password_phc TEXT NOT NULL CHECK(length(password_phc) BETWEEN 40 AND 256),
  disabled INTEGER NOT NULL DEFAULT 0 CHECK(disabled IN(0,1)),
  credential_version INTEGER NOT NULL DEFAULT 1 CHECK(credential_version>=1),
  created_at INTEGER NOT NULL CHECK(created_at>=0)
) STRICT;
CREATE TABLE players (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  account_id INTEGER NOT NULL UNIQUE REFERENCES accounts(id),
  wire_id INTEGER NOT NULL UNIQUE CHECK(wire_id BETWEEN 1 AND 4294967295),
  display_name TEXT NOT NULL CHECK(length(display_name) BETWEEN 3 AND 15),
  profile_version INTEGER NOT NULL DEFAULT 1 CHECK(profile_version>=1)
) STRICT;
CREATE TABLE sessions (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  account_id INTEGER NOT NULL REFERENCES accounts(id),
  token_hash BLOB NOT NULL UNIQUE CHECK(length(token_hash)=32),
  boot_epoch INTEGER NOT NULL CHECK(boot_epoch>=0),
  credential_version INTEGER NOT NULL CHECK(credential_version>=1),
  issued_at INTEGER NOT NULL CHECK(issued_at>=0),
  expires_at INTEGER NOT NULL CHECK(expires_at>issued_at),
  revoked_at INTEGER CHECK(revoked_at IS NULL OR revoked_at>=issued_at)
) STRICT;
CREATE UNIQUE INDEX one_active_session ON sessions(account_id) WHERE revoked_at IS NULL;
CREATE INDEX session_expiry ON sessions(expires_at) WHERE revoked_at IS NULL;
CREATE TABLE admin_audit (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  request_id TEXT NOT NULL UNIQUE CHECK(length(request_id)=64),
  actor TEXT NOT NULL CHECK(length(actor) BETWEEN 1 AND 64),
  operation TEXT NOT NULL,
  account_id INTEGER REFERENCES accounts(id),
  reason TEXT NOT NULL CHECK(length(reason) BETWEEN 1 AND 256),
  at_utc INTEGER NOT NULL CHECK(at_utc>=0),
  details TEXT NOT NULL
) STRICT;
