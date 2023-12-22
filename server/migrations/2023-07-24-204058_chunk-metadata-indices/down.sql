-- This file should undo anything in `up.sql`
DROP INDEX idx_chunk_metadata_private;

DROP INDEX idx_chunk_metadata_oc_file_path;

DROP INDEX idx_chunk_metadata_link;

DROP INDEX idx_chunk_metadata_created_at;

DROP INDEX idx_chunk_metadata_updated_at;

DROP INDEX idx_chunk_collisions_collision_qdrant_id;

DROP INDEX idx_chunk_metadata_id;
