-- This file should undo anything in `up.sql`
-- user_organizations -> organizations
ALTER TABLE user_organizations
DROP CONSTRAINT user_organizations_organization_id_fkey;

ALTER TABLE user_organizations
ADD CONSTRAINT fk_organization_id
FOREIGN KEY (organization_id) REFERENCES organizations(id);