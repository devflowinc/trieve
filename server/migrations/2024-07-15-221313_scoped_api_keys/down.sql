-- This file should undo anything in `up.sql`
ALTER TABLE user_api_key DROP COLUMN IF EXISTS scopes;