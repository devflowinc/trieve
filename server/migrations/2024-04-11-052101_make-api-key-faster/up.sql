-- Your SQL goes here
ALTER TABLE user_api_key ADD COLUMN blake3_hash TEXT;
ALTER TABLE user_api_key ALTER COLUMN api_key_hash DROP NOT NULL;