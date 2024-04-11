-- This file should undo anything in `up.sql`
ALTER TABLE user_api_key ALTER COLUMN api_key_hash SET NOT NULL;
ALTER TABLE user_api_key DROP COLUMN blake3_hash;