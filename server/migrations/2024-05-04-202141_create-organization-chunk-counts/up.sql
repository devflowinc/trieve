-- Your SQL goes here
-- Step 1: Add a new column to store the aggregated chunk counts
ALTER TABLE organization_usage_counts
ADD COLUMN chunk_count INT DEFAULT 0;

-- Step 2: Calculate aggregated chunk counts and store them in a temporary table
WITH AggregatedCounts AS (
    SELECT
        d.organization_id,
        SUM(duc.chunk_count) AS total_chunk_count
    FROM dataset_usage_counts duc
    JOIN datasets d ON duc.dataset_id = d.id
    GROUP BY d.organization_id
)

-- Step 3: Update the organization_usage_counts table with the aggregated chunk counts
-- This will update the chunk_count column in the organization_usage_counts table, but keep the old table so that we can rollback if needed
UPDATE organization_usage_counts o
SET chunk_count = ac.total_chunk_count
FROM AggregatedCounts ac
WHERE o.org_id = ac.organization_id;
