-- Your SQL goes here
ALTER TABLE user_organizations ADD COLUMN IF NOT EXISTS scopes text[];
ALTER TABLE invitations ADD COLUMN IF NOT EXISTS scopes text[];