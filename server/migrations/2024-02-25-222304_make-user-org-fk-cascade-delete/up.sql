-- Your SQL goes here

-- user_organizations -> organizations
ALTER TABLE user_organizations
DROP CONSTRAINT fk_organization_id;

ALTER TABLE user_organizations
ADD CONSTRAINT user_organizations_organization_id_fkey
FOREIGN KEY (organization_id) REFERENCES organizations(id) ON UPDATE CASCADE ON DELETE CASCADE;