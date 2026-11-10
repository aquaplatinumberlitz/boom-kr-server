-- Company design; old sessions keep their original absolute TTL (NULL idle).
ALTER TABLE sessions ADD COLUMN idle_timeout_seconds INTEGER
  CHECK(idle_timeout_seconds IS NULL OR idle_timeout_seconds BETWEEN 1 AND 86400);
ALTER TABLE sessions ADD COLUMN last_seen_at INTEGER
  CHECK(last_seen_at IS NULL OR last_seen_at>=issued_at)
  CHECK((idle_timeout_seconds IS NULL)=(last_seen_at IS NULL));
CREATE INDEX session_idle_expiry ON sessions(last_seen_at)
  WHERE revoked_at IS NULL AND idle_timeout_seconds IS NOT NULL;
