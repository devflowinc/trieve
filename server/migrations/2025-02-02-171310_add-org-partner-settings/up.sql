-- Your SQL goes here
ALTER TABLE organizations ADD COLUMN partner_configuration jsonb NOT NULL DEFAULT '{}';
