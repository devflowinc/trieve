-- This file should undo anything in `up.sql`
DROP TABLE IF EXISTS user_api_key;

ALTER TABLE users ADD COLUMN api_key_hash TEXT;