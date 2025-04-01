-- This file should undo anything in `up.sql`
ALTER TABLE files ADD COLUMN size_mb BIGINT;
UPDATE files SET size_mb = size / 1024;
ALTER TABLE files DROP COLUMN size;
ALTER TABLE files RENAME COLUMN size_mb TO size;

ALTER table organization_usage_counts ADD COLUMN file_storage_mb BIGINT;
UPDATE organization_usage_counts SET file_storage_mb = file_storage / 1024;
ALTER TABLE organization_usage_counts DROP COLUMN file_storage;
ALTER TABLE organization_usage_counts RENAME COLUMN file_storage_mb TO file_storage;

ALTER TABLE stripe_plans ADD COLUMN file_storage_mb BIGINT;
UPDATE stripe_plans SET file_storage_mb = file_storage / 1024;
ALTER TABLE stripe_plans DROP COLUMN file_storage;
ALTER TABLE stripe_plans RENAME COLUMN file_storage_mb TO file_storage;
