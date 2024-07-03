-- Your SQL goes here
DELETE FROM chunk_metadata
WHERE qdrant_point_id IS NULL;

-- Then, alter the table to make the column not nullable
ALTER TABLE chunk_metadata
ALTER COLUMN qdrant_point_id SET NOT NULL;

-- Step 2: Drop the chunk_collisions table
DROP TABLE IF EXISTS chunk_collisions;
