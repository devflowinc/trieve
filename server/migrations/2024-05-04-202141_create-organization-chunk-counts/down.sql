-- This file should undo anything in `up.sql`
ALTER TABLE organization_usage_counts
DROP COLUMN chunk_count;