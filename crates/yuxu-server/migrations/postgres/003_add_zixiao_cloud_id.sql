-- Add Zixiao Labs Cloud Account OAuth linkage to users.
-- zixiao_cloud_id stores the stable subject identifier from the cloud
-- account directory; null for accounts created via username/password
-- registration or via GitHub OAuth.
ALTER TABLE users ADD COLUMN zixiao_cloud_id TEXT;
CREATE UNIQUE INDEX IF NOT EXISTS idx_users_zixiao_cloud_id
    ON users(zixiao_cloud_id) WHERE zixiao_cloud_id IS NOT NULL;
