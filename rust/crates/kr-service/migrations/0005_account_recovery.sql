-- Only the digest of a server-generated single-use recovery code is stored.
CREATE TABLE account_recovery (
  account_id INTEGER PRIMARY KEY REFERENCES accounts(id),
  digest BLOB NOT NULL UNIQUE CHECK(length(digest)=32)
) STRICT;
