-- Your SQL goes here
CREATE INDEX idx_chunk_metadata_dataset_id ON chunk_metadata(dataset_id);

CREATE INDEX idx_chunk_metadata_created_at ON chunk_metadata(created_at);

CREATE INDEX idx_chunk_metadata_time_stamp ON chunk_metadata(time_stamp);

DROP INDEX idx_card_metadata_qdrant_point_id;