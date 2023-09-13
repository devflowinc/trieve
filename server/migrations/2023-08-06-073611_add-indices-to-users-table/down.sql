-- This file should undo anything in `up.sql`
-- Remove index on id column
DROP INDEX IF EXISTS idx_users_id;

-- Remove index on email column
DROP INDEX IF EXISTS idx_users_email;

-- Remove index on hash column
DROP INDEX IF EXISTS idx_users_hash;

-- Remove index on username column
DROP INDEX IF EXISTS idx_users_username;

-- Remove index on website column
DROP INDEX IF EXISTS idx_users_website;

-- Remove index on created_at column
DROP INDEX IF EXISTS idx_users_created_at;

-- Remove index on updated_at column
DROP INDEX IF EXISTS idx_users_updated_at;
