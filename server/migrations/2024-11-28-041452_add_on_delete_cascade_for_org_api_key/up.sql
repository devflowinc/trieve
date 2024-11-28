-- Your SQL goes here
ALTER TABLE organization_api_key
	DROP CONSTRAINT organization_api_key_organization_id_fkey,
	ADD CONSTRAINT organization_api_key_organization_id_fkey
	FOREIGN KEY (organization_id) REFERENCES organizations(id) ON DELETE CASCADE;