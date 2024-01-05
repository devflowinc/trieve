-- This file should undo anything in `up.sql`
DROP TRIGGER IF EXISTS set_default_org_usage_data_trigger ON organizations;

DROP FUNCTION IF EXISTS set_default_org_usage_data;

ALTER TABLE organizations ADD CONSTRAINT organizations_name_key UNIQUE (name);