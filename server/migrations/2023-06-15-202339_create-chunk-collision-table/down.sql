-- This file should undo anything in `up.sql`
DROP TABLE chunk_collisions;
ALTER TABLE chunk_metadata
DROP CONSTRAINT uq_qdrant_point_id;