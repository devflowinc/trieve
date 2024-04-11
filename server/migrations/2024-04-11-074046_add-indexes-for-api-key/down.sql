-- This file should undo anything in `up.sql`
DROP INDEX IF EXISTS user_api_key_api_key_hash_index;
DROP INDEX IF EXISTS user_api_key_blake3_hash_index;