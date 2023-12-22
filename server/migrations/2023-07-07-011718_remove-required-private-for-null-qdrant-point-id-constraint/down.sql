-- This file should undo anything in `up.sql`
ALTER TABLE chunk_metadata
ADD CONSTRAINT qdrant_point_nullable_constraint CHECK (private = true OR qdrant_point_id IS NOT NULL);