-- Your SQL goes here
ALTER TABLE user_api_key ADD COLUMN params JSONB;
ALTER TABLE user_api_key ADD COLUMN expires_at timestamp;