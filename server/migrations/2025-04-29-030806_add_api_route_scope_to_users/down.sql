-- This file should undo anything in `up.sql`
ALTER TABLE user_organizations DROP COLUMN IF EXISTS scopes;
ALTER TABLE invitations DROP COLUMN IF EXISTS scopes;