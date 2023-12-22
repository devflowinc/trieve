-- Your SQL goes here
-- Your SQL goes here
ALTER TABLE chunk_metadata
ADD CONSTRAINT uq_qdrant_point_id UNIQUE (qdrant_point_id);

CREATE TABLE chunk_collisions (
    id UUID PRIMARY KEY,
    chunk_id UUID NOT NULL REFERENCES chunk_metadata (id),
    collision_qdrant_id UUID NOT NULL REFERENCES chunk_metadata (qdrant_point_id)
);

CREATE INDEX idx_chunk_collisions_chunk_id ON chunk_collisions (chunk_id);
