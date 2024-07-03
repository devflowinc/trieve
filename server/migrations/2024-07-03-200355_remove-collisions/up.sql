-- Your SQL goes here
UPDATE chunk_metadata
SET qdrant_point_id = gen_random_uuid() -- Or any default value that makes sense
WHERE qdrant_point_id IS NULL;

-- Then, alter the table to make the column not nullable
ALTER TABLE chunk_metadata
ALTER COLUMN qdrant_point_id SET NOT NULL;

-- Step 2: Drop the chunk_collisions table
DROP TABLE IF EXISTS chunk_collisions;
