-- This file should undo anything in `up.sql`
-- Step 1: Make qdrant_point_id in chunk_metadata nullable again
ALTER TABLE chunk_metadata
ALTER COLUMN qdrant_point_id DROP NOT NULL;

-- Step 2: Recreate the chunk_collisions table
CREATE TABLE chunk_collisions (
    id UUID PRIMARY KEY,
    chunk_id UUID NOT NULL,
    collision_qdrant_id UUID,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);
