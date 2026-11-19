-- Company design: passwordless trial principals, permanently distinct from
-- registered accounts. Their journal references remain valid after disconnect.
CREATE TABLE guest_accounts (
  account_id INTEGER PRIMARY KEY REFERENCES accounts(id),
  boot_epoch INTEGER NOT NULL CHECK(boot_epoch>0),
  expires_at INTEGER NOT NULL CHECK(expires_at>=0)
) STRICT;
CREATE INDEX guest_account_expiry ON guest_accounts(expires_at);
