-- This file should undo anything in `up.sql`
-- Reverse setting chunk_count to 0 for NULL values (not necessary to reverse data update)
-- UPDATE organization_usage_counts
-- SET chunk_count = NULL
-- WHERE chunk_count = 0;

-- Reverse altering the column to SET NOT NULL
ALTER TABLE organization_usage_counts
ALTER COLUMN chunk_count DROP NOT NULL;

-- Reverse the update of chunk_count to the calculated sum (not necessary to reverse data update)
-- UPDATE organization_usage_counts
-- SET chunk_count = NULL;

-- Drop the trigger function
DROP FUNCTION IF EXISTS update_organization_chunk_count;
