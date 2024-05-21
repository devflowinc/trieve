-- Your SQL goes here
ALTER TABLE chunk_metadata DROP COLUMN tag_set;
ALTER TABLE chunk_metadata RENAME COLUMN tag_set_array TO tag_set;
ALTER TABLE chunk_group DROP COLUMN tag_set;
ALTER TABLE chunk_group RENAME COLUMN tag_set_array TO tag_set;
ALTER TABLE files DROP COLUMN tag_set;
ALTER TABLE files RENAME COLUMN tag_set_array TO tag_set;

CREATE INDEX idx_chunk_metadata_tag_set ON chunk_metadata USING gin (tag_set);
CREATE INDEX idx_chunk_group_tag_set ON chunk_group USING gin (tag_set);
CREATE INDEX idx_files_tag_set ON files USING gin (tag_set);
