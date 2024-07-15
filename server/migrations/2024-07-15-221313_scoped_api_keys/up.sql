-- Your SQL goes here
ALTER TABLE user_api_key ADD COLUMN IF NOT EXISTS scopes text[];