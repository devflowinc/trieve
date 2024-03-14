-- Your SQL goes here
-- ALTER TABLE chunk_metadata
DROP INDEX idx_gist;

-- ALTER TABLE chunk_metadata
DROP INDEX idx_link_gin;

-- ALTER TABLE chunk_metadata
DROP INDEX json_gin;

-- ALTER TABLE chunk_metadata
DROP INDEX idx_card_metadata_updated_at;

-- ALTER TABLE chunk_metadata
DROP INDEX idx_card_metadata_created_at;

-- ALTER TABLE chunk_metadata
DROP INDEX card_time_stamp_index;
