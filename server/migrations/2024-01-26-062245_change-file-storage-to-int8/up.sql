-- Your SQL goes here
ALTER TABLE stripe_plans ALTER COLUMN file_storage TYPE BIGINT;

ALTER TABLE organization_usage_counts ALTER COLUMN file_storage TYPE BIGINT;