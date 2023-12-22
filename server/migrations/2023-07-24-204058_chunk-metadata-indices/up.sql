-- Your SQL goes here
CREATE INDEX idx_chunk_metadata_private ON chunk_metadata (private);

CREATE INDEX idx_chunk_metadata_oc_file_path ON chunk_metadata (oc_file_path);

CREATE INDEX idx_chunk_metadata_link ON chunk_metadata (link);

CREATE INDEX idx_chunk_metadata_created_at ON chunk_metadata(created_at);

CREATE INDEX idx_chunk_metadata_updated_at ON chunk_metadata(updated_at);

CREATE INDEX idx_chunk_collisions_collision_qdrant_id ON chunk_collisions (collision_qdrant_id);

CREATE INDEX idx_chunk_metadata_id ON chunk_metadata (id);
