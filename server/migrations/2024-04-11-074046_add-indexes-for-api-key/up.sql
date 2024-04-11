-- Your SQL goes here
CREATE INDEX IF NOT EXISTS user_api_key_api_key_hash_index ON user_api_key (api_key_hash);
CREATE INDEX IF NOT EXISTS user_api_key_blake3_hash_index ON user_api_key (blake3_hash);
