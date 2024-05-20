-- Your SQL goes here
DROP INDEX idx_card_metadata_link;
DROP INDEX idx_card_metadata_oc_file_path;
DROP INDEX idx_chunk_metadata_created_at;
DROP INDEX idx_chunk_metadata_time_stamp;
CREATE INDEX idx_chunk_metadata_json ON chunk_metadata USING gin (metadata jsonb_ops);
