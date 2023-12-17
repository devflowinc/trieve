-- This file should undo anything in `up.sql`
ALTER TABLE user_organizations DROP CONSTRAINT IF EXISTS fk_user_id;
ALTER TABLE user_organizations DROP CONSTRAINT IF EXISTS fk_organization_id;

DROP TABLE IF EXISTS user_organizations;

ALTER TABLE users ADD COLUMN IF NOT EXISTS organization_id UUID;
