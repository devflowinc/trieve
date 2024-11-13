-- This file should undo anything in `up.sql`
ALTER TABLE user_api_key DROP COLUMN params;
ALTER TABLE user_api_key DROP COLUMN expires_at;