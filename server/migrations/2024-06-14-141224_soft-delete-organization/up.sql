-- Your SQL goes here
ALTER TABLE organizations ADD COLUMN deleted INT DEFAULT 0 NOT NULL;
CREATE INDEX IF NOT EXISTS idx_organization_deleted ON organizations (deleted);