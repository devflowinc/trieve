-- Your SQL goes here
CREATE INDEX idx_chunk_metadata_id_dataset_id ON chunk_metadata (dataset_id, id);
