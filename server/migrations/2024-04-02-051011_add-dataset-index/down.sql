-- This file should undo anything in `up.sql`
DROP INDEX idx_chunk_metadata_dataset_id;

DROP INDEX idx_chunk_metadata_created_at;

DROP INDEX idx_chunk_metadata_time_stamp;

CREATE INDEX idx_card_metadata_qdrant_point_id ON chunk_metadata(qdrant_point_id);