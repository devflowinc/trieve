-- This file should undo anything in `up.sql`
ALTER TABLE stripe_plans ALTER COLUMN file_storage TYPE INTEGER;

ALTER TABLE organization_usage_counts ALTER COLUMN file_storage TYPE INTEGER;