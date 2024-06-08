-- This file should undo anything in `up.sql`
ALTER TABLE user_api_key DROP COLUMN IF EXISTS dataset_ids;
ALTER TABLE user_api_key DROP COLUMN IF EXISTS organization_ids;