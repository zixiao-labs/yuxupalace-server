-- Add GitHub OAuth linkage to users.
-- github_id stores the numeric GitHub user id as a string; null for accounts
-- created via username/password registration.
ALTER TABLE users ADD COLUMN github_id TEXT;
CREATE UNIQUE INDEX IF NOT EXISTS idx_users_github_id ON users(github_id) WHERE github_id IS NOT NULL;
