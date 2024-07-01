-- Your SQL goes here
DROP TABLE chunk_collisions;
ALTER TABLE chunk_metadata
ALTER COLUMN qdrant_point_id SET NOT NULL;