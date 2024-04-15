-- Your SQL goes here
ALTER TABLE datasets ADD COLUMN tracking_id TEXT NULL;
ALTER TABLE datasets ADD CONSTRAINT unique_organization_tracking_id UNIQUE (organization_id, tracking_id);
CREATE INDEX datasets_tracking_id_idx ON datasets (tracking_id);