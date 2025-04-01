-- files

ALTER TABLE files ADD COLUMN size_bytes BIGINT NOT NULL DEFAULT 0;
UPDATE files SET size_bytes = size * 1024;
ALTER TABLE files DROP COLUMN size;
ALTER TABLE files RENAME COLUMN size_bytes TO size;

-- organization_usage_counts

ALTER table organization_usage_counts ADD COLUMN file_storage_bytes BIGINT NOT NULL DEFAULT 0;
UPDATE organization_usage_counts SET file_storage_bytes = file_storage * 1024;
ALTER TABLE organization_usage_counts DROP COLUMN file_storage;
ALTER TABLE organization_usage_counts RENAME COLUMN file_storage_bytes TO file_storage;

--  stripe_plans
ALTER TABLE stripe_plans ADD COLUMN file_storage_bytes BIGINT NOT NULL DEFAULT 0;
UPDATE stripe_plans SET file_storage_bytes = file_storage * 1024;
ALTER TABLE stripe_plans DROP COLUMN file_storage;
ALTER TABLE stripe_plans RENAME COLUMN file_storage_bytes TO file_storage;
