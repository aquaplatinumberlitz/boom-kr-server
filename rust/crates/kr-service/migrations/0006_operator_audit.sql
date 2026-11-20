-- Existing rows were published only on success, in the mutation transaction.
ALTER TABLE admin_audit ADD COLUMN target_kind TEXT;
ALTER TABLE admin_audit ADD COLUMN target_id TEXT;
ALTER TABLE admin_audit ADD COLUMN result TEXT NOT NULL DEFAULT 'ok';
UPDATE admin_audit SET target_kind='account', target_id=CAST(account_id AS TEXT)
 WHERE account_id IS NOT NULL;
CREATE INDEX admin_audit_target ON admin_audit(target_kind,target_id,id);
