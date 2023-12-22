-- Your SQL goes here
--Add private column to chunk_metadata
ALTER TABLE chunk_metadata
ADD COLUMN private BOOLEAN NOT NULL DEFAULT false;

--make qdrant_point_id nullable
ALTER TABLE chunk_metadata
ALTER COLUMN qdrant_point_id DROP NOT NULL;

-- Add the CHECK constraint to the table
ALTER TABLE chunk_metadata
ADD CONSTRAINT qdrant_point_nullable_constraint CHECK (private = true OR qdrant_point_id IS NOT NULL);
